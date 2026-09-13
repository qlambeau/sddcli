use gherkin::{Feature, GherkinEnv, StepType};
use pulldown_cmark::Parser;
use serde::de::DeserializeOwned;
use serde_yaml::Value;

use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, ChecklistItem, Diagnostic, DocumentSnapshot,
    FeatureSnapshot, Heading, Location, Metadata, MetadataValue, ReleaseFeature, ReleaseSnapshot,
    ScenarioCoverage, Severity, SourceLine,
};

const FRONTMATTER_SEPARATOR: &str = "---";

/// Parses one artifact into the normalized domain boundary.
#[must_use]
pub fn parse_artifact(path: ArtifactPath, kind: ArtifactKind, contents: &str) -> ArtifactSnapshot {
    let lines = source_lines(contents);
    let (metadata, mut diagnostics) = if kind == ArtifactKind::Gherkin {
        (Metadata::new(), Vec::new())
    } else {
        parse_frontmatter(&path, kind, contents)
    };
    let document = if kind == ArtifactKind::Gherkin {
        parse_gherkin(&path, contents, lines, &mut diagnostics)
    } else {
        parse_markdown(&path, contents, &lines, kind, &metadata, &mut diagnostics)
    };
    ArtifactSnapshot::new(path, kind, metadata, document).with_parser_diagnostics(diagnostics)
}

fn parse_frontmatter(
    path: &ArtifactPath,
    kind: ArtifactKind,
    contents: &str,
) -> (Metadata, Vec<Diagnostic>) {
    let lines: Vec<&str> = contents.lines().collect();
    let Some(first) = lines.first() else {
        return missing_frontmatter(path, kind, 1);
    };
    if first.trim() != FRONTMATTER_SEPARATOR {
        return missing_frontmatter(path, kind, 1);
    }
    let Some(end) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| (line.trim() == FRONTMATTER_SEPARATOR).then_some(index))
    else {
        return (
            Metadata::new(),
            vec![parser_diagnostic(
                path,
                kind,
                "FRONTMATTER_PARSE",
                Some(Location::new(1, None)),
                "frontmatter is not closed by a separator",
                "add a closing `---` line after the YAML frontmatter",
            )],
        );
    };

    let yaml = lines.get(1..end).map_or_else(String::new, |lines| lines.join("\n"));
    let value = match decode_yaml::<Value>(&yaml) {
        Ok(value) => value,
        Err(error) => {
            return (
                Metadata::new(),
                vec![parser_diagnostic(
                    path,
                    kind,
                    "FRONTMATTER_PARSE",
                    Some(Location::new(2, None)),
                    format!("frontmatter YAML could not be parsed: {error}"),
                    "repair the YAML frontmatter syntax",
                )],
            );
        }
    };
    let Value::Mapping(mapping) = value else {
        return (
            Metadata::new(),
            vec![parser_diagnostic(
                path,
                kind,
                "FRONTMATTER_SHAPE",
                Some(Location::new(2, None)),
                "frontmatter must be a YAML mapping",
                "put named metadata fields below the opening `---` line",
            )],
        );
    };

    let mut metadata = Metadata::new();
    let mut diagnostics = Vec::new();
    for (key, value) in mapping {
        let Some(name) = key.as_str() else {
            diagnostics.push(parser_diagnostic(
                path,
                kind,
                "FRONTMATTER_KEY",
                Some(Location::new(2, None)),
                "frontmatter contains a non-string field name",
                "use a plain string as every frontmatter field name",
            ));
            continue;
        };
        metadata.insert(name, normalized_value(value));
    }
    (metadata, diagnostics)
}

fn decode_yaml<T>(yaml: &str) -> Result<T, serde_yaml::Error>
where
    T: DeserializeOwned,
{
    serde_yaml::from_str(yaml)
}

fn normalized_value(value: Value) -> MetadataValue {
    match value {
        Value::Null => MetadataValue::Null,
        Value::String(value) => MetadataValue::Scalar(value),
        Value::Bool(value) => MetadataValue::Scalar(value.to_string()),
        Value::Number(value) => MetadataValue::Scalar(value.to_string()),
        Value::Sequence(values) => {
            let strings: Option<Vec<String>> = values
                .into_iter()
                .map(|value| match value {
                    Value::String(value) => Some(value),
                    Value::Bool(value) => Some(value.to_string()),
                    Value::Number(value) => Some(value.to_string()),
                    Value::Null | Value::Sequence(_) | Value::Mapping(_) | Value::Tagged(_) => None,
                })
                .collect();
            strings.map_or(MetadataValue::Mapping, MetadataValue::Sequence)
        }
        Value::Mapping(_) | Value::Tagged(_) => MetadataValue::Mapping,
    }
}

fn parse_markdown(
    path: &ArtifactPath,
    contents: &str,
    lines: &[SourceLine],
    kind: ArtifactKind,
    metadata: &Metadata,
    diagnostics: &mut Vec<Diagnostic>,
) -> DocumentSnapshot {
    let headings = lines.iter().filter_map(parse_heading).collect::<Vec<_>>();
    let checklist_items = lines.iter().filter_map(parse_checklist).collect::<Vec<_>>();
    let _ = Parser::new(contents).count();
    let document = DocumentSnapshot::new(lines.to_owned(), headings, checklist_items, None);
    if kind == ArtifactKind::Release {
        return document.with_release(parse_release(path, lines, metadata, diagnostics));
    }
    document
}

fn parse_release(
    path: &ArtifactPath,
    lines: &[SourceLine],
    metadata: &Metadata,
    diagnostics: &mut Vec<Diagnostic>,
) -> ReleaseSnapshot {
    let features = parse_release_features(path, lines, diagnostics);
    let has_verification_evidence = has_verification_evidence(lines);
    let has_release_commit = has_release_commit(metadata);
    ReleaseSnapshot::new(features, has_verification_evidence, has_release_commit)
}

fn parse_release_features(
    path: &ArtifactPath,
    lines: &[SourceLine],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ReleaseFeature> {
    let mut in_feature_table = false;
    let mut features = Vec::new();
    for line in lines {
        if let Some(heading) = parse_heading(line) {
            in_feature_table = heading.text().eq_ignore_ascii_case("included features");
        }
        if !in_feature_table {
            continue;
        }
        let trimmed = line.text().trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cells = trimmed.trim_matches('|').split('|').map(str::trim).collect::<Vec<_>>();
        let Some(first) = cells.first() else { continue };
        if first.eq_ignore_ascii_case("feature id") || is_separator_row(&cells) {
            continue;
        }
        if let Some(feature) = parse_release_feature_row(path, line, &cells, diagnostics) {
            features.push(feature);
        }
    }
    features
}

fn parse_release_feature_row(
    path: &ArtifactPath,
    line: &SourceLine,
    cells: &[&str],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ReleaseFeature> {
    let story_id = cells.get(2).copied().unwrap_or_default().trim_matches('`');
    let status = cells.get(3).copied().unwrap_or_default().trim_matches('`');
    if cells.len() < 4 || !valid_story_id(story_id) || status.is_empty() {
        diagnostics.push(parser_diagnostic(
            path,
            ArtifactKind::Release,
            "RELEASE_FEATURE_ROW",
            Some(Location::new(line.line(), None)),
            "included features contains a malformed feature row",
            "record a US-NNN user story and lifecycle status in every feature row",
        ));
        return None;
    }
    Some(ReleaseFeature::new(story_id, status))
}

fn is_separator_row(cells: &[&str]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .all(|cell| cell.chars().all(|character| character == '-' || character == ' '))
}

fn has_verification_evidence(lines: &[SourceLine]) -> bool {
    let Some((index, level)) = lines.iter().enumerate().find_map(|(index, line)| {
        let heading = parse_heading(line)?;
        heading
            .text()
            .eq_ignore_ascii_case("verification evidence")
            .then_some((index, heading.level()))
    }) else {
        return false;
    };
    lines
        .iter()
        .skip(index + 1)
        .take_while(|line| parse_heading(line).is_none_or(|heading| heading.level() > level))
        .any(|line| {
            let trimmed = line.text().trim();
            let value = trimmed.strip_prefix('-').map(str::trim).unwrap_or_default();
            !value.is_empty() && !value.to_ascii_lowercase().contains("tbd")
        })
}

fn has_release_commit(metadata: &Metadata) -> bool {
    match metadata.get("commit") {
        Some(MetadataValue::Scalar(value)) => {
            let value = value.trim();
            !value.is_empty() && value != "<git-hash>" && !value.eq_ignore_ascii_case("TBD")
        }
        _ => false,
    }
}

fn valid_story_id(value: &str) -> bool {
    let Some(number) = value.strip_prefix("US-") else { return false };
    number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_gherkin(
    path: &ArtifactPath,
    contents: &str,
    lines: Vec<SourceLine>,
    diagnostics: &mut Vec<Diagnostic>,
) -> DocumentSnapshot {
    let (parent, status) = parse_headers(&lines);
    let feature = match Feature::parse(contents, GherkinEnv::default()) {
        Ok(feature) => Some(feature_snapshot(feature, parent, status)),
        Err(error) => {
            diagnostics.push(parser_diagnostic(
                path,
                ArtifactKind::Gherkin,
                "GHERKIN_PARSE",
                None,
                format!("Gherkin content could not be parsed: {error}"),
                "repair the Feature, Scenario, and step syntax",
            ));
            None
        }
    };
    DocumentSnapshot::new(lines, Vec::new(), Vec::new(), feature)
}

fn parse_headers(lines: &[SourceLine]) -> (Option<String>, Option<String>) {
    let parent = lines.iter().find_map(|line| {
        line.text().trim().strip_prefix("# parent:").map(|value| value.trim().to_string())
    });
    let status = lines.iter().find_map(|line| {
        line.text().trim().strip_prefix("# status:").map(|value| value.trim().to_string())
    });
    (parent, status)
}

fn feature_snapshot(
    feature: Feature,
    parent: Option<String>,
    status: Option<String>,
) -> FeatureSnapshot {
    let scenarios =
        feature.scenarios.iter().chain(feature.rules.iter().flat_map(|rule| rule.scenarios.iter()));
    let mut scenario_count = 0;
    let mut step_presence = (false, false, false);
    let mut coverage = std::collections::BTreeSet::new();
    for scenario in scenarios {
        scenario_count += 1;
        if let Some(category) = ScenarioCoverage::from_name(&scenario.name) {
            coverage.insert(category);
        }
        for step in &scenario.steps {
            match step.ty {
                StepType::Given => step_presence.0 = true,
                StepType::When => step_presence.1 = true,
                StepType::Then => step_presence.2 = true,
            }
        }
    }
    FeatureSnapshot::new_with_coverage(
        parent,
        status,
        Some(feature.name),
        scenario_count,
        step_presence.0,
        step_presence.1,
        step_presence.2,
        coverage,
    )
}

fn source_lines(contents: &str) -> Vec<SourceLine> {
    contents.lines().enumerate().map(|(index, line)| SourceLine::new(index + 1, line)).collect()
}

fn parse_heading(line: &SourceLine) -> Option<Heading> {
    let trimmed = line.text().trim_start();
    let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let text = trimmed.get(level..)?.trim_start();
    if text.is_empty() || !trimmed.as_bytes().get(level).is_some_and(u8::is_ascii_whitespace) {
        return None;
    }
    Some(Heading::new(u8::try_from(level).ok()?, text.trim_end_matches('#').trim(), line.line()))
}

fn parse_checklist(line: &SourceLine) -> Option<ChecklistItem> {
    let trimmed = line.text().trim_start();
    let marker = ["- [", "* [", "+ ["].iter().find_map(|prefix| trimmed.strip_prefix(prefix))?;
    let (checked, remainder) = match marker.chars().next()? {
        'x' | 'X' => (true, marker.get(1..)?),
        ' ' => (false, marker.get(1..)?),
        _ => return None,
    };
    let text = remainder.strip_prefix("] ")?;
    Some(ChecklistItem::new(checked, text, line.line()))
}

fn missing_frontmatter(
    path: &ArtifactPath,
    kind: ArtifactKind,
    line: usize,
) -> (Metadata, Vec<Diagnostic>) {
    (
        Metadata::new(),
        vec![parser_diagnostic(
            path,
            kind,
            "FRONTMATTER_REQUIRED",
            Some(Location::new(line, None)),
            "active Markdown artifact has no YAML frontmatter",
            "add YAML frontmatter between opening and closing `---` lines",
        )],
    )
}

fn parser_diagnostic(
    path: &ArtifactPath,
    kind: ArtifactKind,
    rule: &str,
    location: Option<Location>,
    message: impl Into<String>,
    remediation: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        path.clone(),
        location,
        format!("ARTIFACT.{}.{}", kind.token().to_ascii_uppercase(), rule),
        Severity::Error,
        message,
        remediation,
    )
}
