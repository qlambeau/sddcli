use crate::{
    ArtifactKind, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Heading, Location, MetadataValue,
    Severity,
};

const ALLOWED_STATUSES: [&str; 7] =
    ["draft", "in-review", "approved", "implemented", "archived", "released", "superseded"];

/// Applies every pure structural rule to a normalized artifact.
#[must_use]
pub fn validate_artifact(snapshot: &ArtifactSnapshot) -> Vec<Diagnostic> {
    let mut diagnostics = snapshot.parser_diagnostics().to_vec();
    let kind = snapshot.kind();
    let path = snapshot.path();

    if kind == ArtifactKind::Gherkin {
        validate_gherkin(snapshot.document(), path, &mut diagnostics);
    } else {
        validate_frontmatter(snapshot, &mut diagnostics);
        validate_document(kind, snapshot.document(), path, &mut diagnostics);
    }
    validate_markers(snapshot, &mut diagnostics);
    diagnostics.sort_by(Diagnostic::stable_cmp);
    diagnostics
}

fn validate_frontmatter(snapshot: &ArtifactSnapshot, diagnostics: &mut Vec<Diagnostic>) {
    let kind = snapshot.kind();
    let metadata = snapshot.metadata();
    let path = snapshot.path();
    for field in required_fields(kind) {
        if metadata.get(field).is_none() {
            add(
                diagnostics,
                path,
                "METADATA_REQUIRED",
                format!("required metadata field `{field}` is missing"),
                format!("add `{field}` to the {kind:?} frontmatter"),
            );
        }
    }

    validate_field_shapes(kind, metadata, path, diagnostics);
    validate_type(kind, metadata.get("type"), path, diagnostics);
    validate_status(metadata.get("status"), path, diagnostics);
    validate_id(kind, metadata.get("id"), path, diagnostics);
    validate_references(kind, metadata, path, diagnostics);
}

fn validate_field_shapes(
    kind: ArtifactKind,
    metadata: &crate::Metadata,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for &field in required_fields(kind) {
        let Some(value) = metadata.get(field) else {
            continue;
        };
        let shape_matches = field_shape_matches(field, value);
        if !shape_matches {
            let expected = expected_shape(field);
            add(
                diagnostics,
                path,
                "METADATA_SHAPE",
                format!("metadata `{field}` must be {expected}"),
                format!("change `{field}` to {expected} in frontmatter"),
            );
        }
    }
}

fn field_shape_matches(field: &str, value: &MetadataValue) -> bool {
    if matches!(field, "depends_on" | "requires" | "blockers" | "related") {
        return matches!(value, MetadataValue::Sequence(_));
    }
    if matches!(field, "parent" | "supersedes" | "superseded_by") {
        return matches!(value, MetadataValue::Null | MetadataValue::Scalar(_));
    }
    matches!(value, MetadataValue::Scalar(_))
}

fn expected_shape(field: &str) -> &'static str {
    if matches!(field, "depends_on" | "requires" | "blockers" | "related") {
        "a YAML sequence"
    } else {
        "a YAML scalar"
    }
}

fn validate_type(
    kind: ArtifactKind,
    value: Option<&MetadataValue>,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(expected) = kind.expected_type() else {
        return;
    };
    let actual = scalar(value);
    if actual != Some(expected) {
        add(
            diagnostics,
            path,
            "TYPE_VALUE",
            format!("metadata `type` must be `{expected}`"),
            format!("set `type: {expected}`"),
        );
    }
}

fn validate_status(
    value: Option<&MetadataValue>,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(actual) = scalar(value) else {
        add(
            diagnostics,
            path,
            "STATUS_VALUE",
            "metadata `status` must be a lifecycle value",
            "set status to one of draft, in-review, approved, implemented, archived, or released",
        );
        return;
    };
    if !ALLOWED_STATUSES.contains(&actual) {
        add(
            diagnostics,
            path,
            "STATUS_VALUE",
            format!("metadata `status` has unsupported value `{actual}`"),
            "use an allowed lifecycle status",
        );
    }
}

fn validate_id(
    kind: ArtifactKind,
    value: Option<&MetadataValue>,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(prefix) = kind.id_prefix() else {
        return;
    };
    let Some(actual) = scalar(value) else {
        add(
            diagnostics,
            path,
            "IDENTITY_VALUE",
            format!("metadata `id` must use the {prefix}-NNN format"),
            format!("set `id` to a {prefix}-NNN identifier"),
        );
        return;
    };
    if crate::ArtifactId::try_new(kind, actual).is_err() {
        add(
            diagnostics,
            path,
            "IDENTITY_VALUE",
            format!("metadata `id` must use the {prefix}-NNN format"),
            format!("set `id` to a {prefix}-NNN identifier"),
        );
    }
}

fn validate_references(
    kind: ArtifactKind,
    metadata: &crate::Metadata,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for field in [
        "parent",
        "epic",
        "feature",
        "depends_on",
        "requires",
        "related",
        "supersedes",
        "superseded_by",
    ] {
        let Some(value) = metadata.get(field) else {
            continue;
        };
        for reference in value_strings(value) {
            if is_unresolved(reference) || reference.is_empty() {
                continue;
            }
            if field == "feature" {
                continue;
            }
            if !valid_reference(reference) {
                add(
                    diagnostics,
                    path,
                    "REFERENCE_SYNTAX",
                    format!("metadata `{field}` contains invalid local reference `{reference}`"),
                    "use a PREFIX-NNN local artifact reference",
                );
            }
        }
    }
    if kind == ArtifactKind::Prd && !is_null(metadata.get("parent")) {
        add(
            diagnostics,
            path,
            "REFERENCE_PARENT",
            "a PRD `parent` field must be null",
            "set the PRD parent field to null",
        );
    }
    for (field, prefix) in expected_reference_prefixes(kind) {
        let Some(value) = scalar(metadata.get(field)) else {
            continue;
        };
        if !valid_identifier_text(value, prefix) {
            add(
                diagnostics,
                path,
                "REFERENCE_PARENT",
                format!("metadata `{field}` must contain a {prefix}-NNN identifier"),
                format!("set `{field}` to a {prefix}-NNN identifier"),
            );
        }
    }
}

fn expected_reference_prefixes(kind: ArtifactKind) -> &'static [(&'static str, &'static str)] {
    match kind {
        ArtifactKind::Prd | ArtifactKind::Adr | ArtifactKind::Gherkin => &[],
        ArtifactKind::Epic => &[("parent", "PRD")],
        ArtifactKind::UserStory => &[("parent", "PRD"), ("epic", "EPIC")],
        ArtifactKind::Requirements | ArtifactKind::Design | ArtifactKind::Task => {
            &[("parent", "US")]
        }
    }
}

fn validate_document(
    kind: ArtifactKind,
    document: &DocumentSnapshot,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for required in required_headings(kind) {
        if let Some(heading) =
            document.headings().iter().find(|heading| heading_matches(heading, required))
        {
            if let Some(checklist_name) = checklist_heading(kind) {
                let next_heading_line = document
                    .headings()
                    .iter()
                    .filter(|candidate| {
                        candidate.line() > heading.line() && candidate.level() <= heading.level()
                    })
                    .map(Heading::line)
                    .min();
                let has_item = document.checklist_items().iter().any(|item| {
                    item.line() > heading.line()
                        && next_heading_line.is_none_or(|line| item.line() < line)
                });
                if checklist_name == *required && !has_item {
                    add(
                        diagnostics,
                        path,
                        "CHECKLIST_ITEM",
                        format!("checklist section `{required}` has no checklist items"),
                        "add at least one checklist item to the required section",
                    );
                }
            }
        } else {
            add(
                diagnostics,
                path,
                "SECTION_REQUIRED",
                format!("required section `{required}` is missing"),
                format!("add a Markdown heading named `{required}`"),
            );
        }
    }
    if let Some(checklist_heading) = checklist_heading(kind)
        && !document.headings().iter().any(|heading| heading_matches(heading, checklist_heading))
    {
        add(
            diagnostics,
            path,
            "CHECKLIST_SECTION",
            format!("required checklist section `{checklist_heading}` is missing"),
            format!("add a `{checklist_heading}` heading and checklist items"),
        );
    }
}

fn validate_gherkin(
    document: &DocumentSnapshot,
    path: &crate::ArtifactPath,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(feature) = document.feature() else {
        if !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("GHERKIN_PARSE"))
        {
            add(
                diagnostics,
                path,
                "GHERKIN_PARSE",
                "the feature file could not be parsed",
                "provide valid Gherkin with a Feature and Scenario",
            );
        }
        return;
    };
    match feature.parent() {
        Some(parent) if valid_identifier_text(parent, "US") => {}
        _ => add(
            diagnostics,
            path,
            "HEADER_PARENT",
            "the `# parent:` header must contain a US-NNN identifier",
            "add `# parent: US-NNN` with the owning story identifier",
        ),
    }
    match feature.status() {
        Some(status) if ALLOWED_STATUSES.contains(&status) => {}
        _ => add(
            diagnostics,
            path,
            "HEADER_STATUS",
            "the `# status:` header must contain an allowed lifecycle value",
            "set `# status:` to draft, in-review, approved, implemented, archived, or released",
        ),
    }
    if feature.name().is_none() {
        add(
            diagnostics,
            path,
            "FEATURE_REQUIRED",
            "the feature file must contain a Feature",
            "add a Gherkin Feature declaration",
        );
    }
    if feature.scenario_count() == 0 {
        add(
            diagnostics,
            path,
            "SCENARIO_REQUIRED",
            "the feature file must contain at least one Scenario",
            "add a Gherkin Scenario with Given, When, and Then steps",
        );
    }
    for (present, name) in
        [(feature.has_given(), "Given"), (feature.has_when(), "When"), (feature.has_then(), "Then")]
    {
        if !present {
            add(
                diagnostics,
                path,
                "STEP_REQUIRED",
                format!("the feature file has no `{name}` step"),
                format!("add a `{name}` step to a scenario"),
            );
        }
    }
}

fn validate_markers(snapshot: &ArtifactSnapshot, diagnostics: &mut Vec<Diagnostic>) {
    for (name, value) in snapshot.metadata().iter() {
        for text in value_strings(value) {
            if is_unresolved(text) {
                add_at(
                    diagnostics,
                    snapshot.path(),
                    "PLACEHOLDER_UNRESOLVED",
                    format!("metadata `{name}` contains unresolved value `{text}`"),
                    "replace the unresolved value with concrete artifact content",
                    None,
                );
            }
            if is_template_marker(text) {
                add_at(
                    diagnostics,
                    snapshot.path(),
                    "PLACEHOLDER_TEMPLATE",
                    format!("metadata `{name}` contains an unreplaced template marker"),
                    "replace the template marker with a concrete value",
                    None,
                );
            }
        }
    }
    for line in snapshot.document().lines() {
        if is_unresolved(line.text()) {
            add_at(
                diagnostics,
                snapshot.path(),
                "PLACEHOLDER_UNRESOLVED",
                "document content contains an unresolved value marker",
                "replace the unresolved marker with concrete artifact content",
                Some(Location::new(line.line(), None)),
            );
        }
        if is_template_marker(line.text()) {
            add_at(
                diagnostics,
                snapshot.path(),
                "PLACEHOLDER_TEMPLATE",
                "document content contains an unreplaced template marker",
                "replace the template marker with concrete artifact content",
                Some(Location::new(line.line(), None)),
            );
        }
    }
}

fn required_fields(kind: ArtifactKind) -> &'static [&'static str] {
    match kind {
        ArtifactKind::Prd => &[
            "id",
            "title",
            "type",
            "scope",
            "status",
            "created",
            "updated",
            "owner",
            "parent",
            "supersedes",
            "related",
        ],
        ArtifactKind::Epic
        | ArtifactKind::Requirements
        | ArtifactKind::Design
        | ArtifactKind::Task => &[
            "id",
            "title",
            "type",
            "status",
            "created",
            "updated",
            "owner",
            "parent",
            "depends_on",
            "requires",
            "blockers",
            "related",
        ],
        ArtifactKind::UserStory => &[
            "id",
            "title",
            "type",
            "status",
            "created",
            "updated",
            "owner",
            "parent",
            "epic",
            "feature",
            "depends_on",
            "requires",
            "blockers",
            "related",
        ],
        ArtifactKind::Gherkin => &[],
        ArtifactKind::Adr => &[
            "id",
            "title",
            "type",
            "status",
            "created",
            "updated",
            "owner",
            "supersedes",
            "superseded_by",
            "related",
        ],
    }
}

fn required_headings(kind: ArtifactKind) -> &'static [&'static str] {
    match kind {
        ArtifactKind::Prd => &[
            "product requirements document",
            "vision and problem",
            "target personas and journeys",
            "success metrics",
            "functional scope and epics",
            "non-functional requirements",
            "assumptions and out of scope",
            "open questions",
            "decision log",
            "review checklist",
        ],
        ArtifactKind::Epic => &[
            "epic brief",
            "outcome statement",
            "capability boundaries",
            "candidate vertical slices",
            "success criteria",
            "dependencies",
            "open questions",
            "readiness checklist",
        ],
        ArtifactKind::UserStory => &[
            "user story",
            "story card",
            "context and value",
            "business rules",
            "examples",
            "acceptance criteria",
            "scope boundaries",
            "dependencies",
            "open questions",
            "invest check",
        ],
        ArtifactKind::Requirements => &[
            "requirements",
            "purpose and actors",
            "preconditions",
            "inputs and outputs",
            "functional requirements",
            "postconditions and invariants",
            "edge and failure behavior",
            "quality requirements",
            "traceability",
        ],
        ArtifactKind::Design => &[
            "design",
            "context and constraints",
            "proposed design",
            "components and responsibilities",
            "interfaces and contracts",
            "data and state flow",
            "security, performance, and operations",
            "alternatives considered",
            "risks and open decisions",
            "verification approach",
        ],
        ArtifactKind::Adr => {
            &["context", "decision", "alternatives considered", "consequences", "follow-up actions"]
        }
        ArtifactKind::Task => &[
            "tasks",
            "implementation approach",
            "ordered tasks",
            "test and verification plan",
            "rollout and recovery",
            "definition of done",
        ],
        ArtifactKind::Gherkin => &[],
    }
}

fn checklist_heading(kind: ArtifactKind) -> Option<&'static str> {
    match kind {
        ArtifactKind::Prd => Some("review checklist"),
        ArtifactKind::Epic => Some("readiness checklist"),
        ArtifactKind::UserStory => Some("invest check"),
        ArtifactKind::Task => Some("definition of done"),
        _ => None,
    }
}

fn heading_matches(heading: &Heading, required: &str) -> bool {
    normalize_heading(heading.text()) == required
}

fn normalize_heading(value: &str) -> String {
    let without_number = value.trim().trim_start_matches(|character: char| {
        character.is_ascii_digit() || character == '.' || character == ' '
    });
    without_number.to_ascii_lowercase()
}

fn scalar(value: Option<&MetadataValue>) -> Option<&str> {
    match value {
        Some(MetadataValue::Scalar(value)) => Some(value),
        _ => None,
    }
}

fn value_strings(value: &MetadataValue) -> Vec<&str> {
    match value {
        MetadataValue::Scalar(value) => vec![value],
        MetadataValue::Sequence(values) => values.iter().map(String::as_str).collect(),
        MetadataValue::Null | MetadataValue::Mapping => Vec::new(),
    }
}

fn is_null(value: Option<&MetadataValue>) -> bool {
    matches!(value, Some(MetadataValue::Null))
}

fn is_unresolved(value: &str) -> bool {
    value.split_whitespace().any(|word| word.eq_ignore_ascii_case("TBD"))
}

fn is_template_marker(value: &str) -> bool {
    value.contains("YYYY-MM-DD")
        || value.contains("{{")
        || value.contains("}}")
        || value.split_whitespace().any(|word| {
            word == "NNN" || word.ends_with("-NNN") || word.starts_with('<') && word.ends_with('>')
        })
}

fn valid_reference(value: &str) -> bool {
    ["PRD", "EPIC", "US", "REQ", "DES", "ADR", "TASK", "CHART", "DB", "TABLE", "OBS", "REL"]
        .iter()
        .any(|prefix| valid_identifier_text(value, prefix))
}

fn valid_identifier_text(value: &str, prefix: &str) -> bool {
    let Some(number) = value.strip_prefix(prefix).and_then(|value| value.strip_prefix('-')) else {
        return false;
    };
    number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn add(
    diagnostics: &mut Vec<Diagnostic>,
    path: &crate::ArtifactPath,
    rule: &str,
    message: impl Into<String>,
    remediation: impl Into<String>,
) {
    add_at(diagnostics, path, rule, message, remediation, None);
}

fn add_at(
    diagnostics: &mut Vec<Diagnostic>,
    path: &crate::ArtifactPath,
    rule: &str,
    message: impl Into<String>,
    remediation: impl Into<String>,
    location: Option<Location>,
) {
    let kind = ArtifactKind::from_path(path.as_str())
        .map_or("unknown", ArtifactKind::token)
        .to_ascii_uppercase();
    diagnostics.push(Diagnostic::new(
        path.clone(),
        location,
        format!("ARTIFACT.{kind}.{rule}"),
        Severity::Error,
        message,
        remediation,
    ));
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;
    use crate::{
        ArtifactPath, ArtifactResult, ArtifactStatus, DocumentSnapshot, Heading, Metadata,
        OverallStatus, SourceLine, ValidationReport,
    };

    fn valid_snapshot(kind: ArtifactKind, status: &str) -> ArtifactSnapshot {
        let mut metadata = Metadata::new();
        for &field in required_fields(kind) {
            match field {
                "id" => metadata.insert_scalar(field, expected_id(kind)),
                "type" => metadata.insert_scalar(field, kind.expected_type().unwrap_or_default()),
                "status" => metadata.insert_scalar(field, status),
                "parent" if kind == ArtifactKind::Prd => {
                    metadata.insert(field, MetadataValue::Null);
                }
                "parent" => {
                    let parent = match kind {
                        ArtifactKind::Epic | ArtifactKind::UserStory => "PRD-001",
                        _ => "US-001",
                    };
                    metadata.insert_scalar(field, parent);
                }
                "epic" => metadata.insert_scalar(field, "EPIC-001"),
                "feature" => metadata.insert_scalar(field, "feature-name"),
                "depends_on" | "requires" | "related" | "blockers" => {
                    metadata.insert_sequence(field, Vec::<String>::new());
                }
                "supersedes" | "superseded_by" => metadata.insert(field, MetadataValue::Null),
                "scope" => metadata.insert_scalar(field, "project"),
                "created" | "updated" => metadata.insert_scalar(field, "2026-09-04"),
                _ => metadata.insert_scalar(field, "concrete-value"),
            }
        }
        let headings = required_headings(kind)
            .iter()
            .enumerate()
            .map(|(index, heading)| Heading::new(2, *heading, index + 1))
            .collect();
        let checklist_items = checklist_heading(kind)
            .map(|_| vec![crate::ChecklistItem::new(true, "complete", 20)])
            .unwrap_or_default();
        let lines = vec![SourceLine::new(1, "concrete artifact")];
        let path = fixture_path(path_for(kind));
        ArtifactSnapshot::new(
            path,
            kind,
            metadata,
            DocumentSnapshot::new(lines, headings, checklist_items, None),
        )
    }

    fn expected_id(kind: ArtifactKind) -> String {
        format!("{}-001", kind.id_prefix().unwrap_or("NO"))
    }

    fn path_for(kind: ArtifactKind) -> String {
        match kind {
            ArtifactKind::Prd => "specs/prds/PRD-001.md".to_string(),
            ArtifactKind::Epic => "specs/prds/PRD-001-epics/EPIC-001.md".to_string(),
            ArtifactKind::UserStory => "specs/feature/user-story.md".to_string(),
            ArtifactKind::Gherkin => "specs/feature/scenarios.feature".to_string(),
            ArtifactKind::Requirements => "specs/feature/requirements.md".to_string(),
            ArtifactKind::Design => "specs/feature/design.md".to_string(),
            ArtifactKind::Adr => "specs/adr/ADR-001.md".to_string(),
            ArtifactKind::Task => "specs/feature/tasks.md".to_string(),
        }
    }

    fn fixture_path(value: String) -> ArtifactPath {
        match ArtifactPath::try_new(value) {
            Ok(path) => path,
            Err(_) => std::process::abort(),
        }
    }

    #[rstest]
    #[case(ArtifactKind::Prd)]
    #[case(ArtifactKind::Epic)]
    #[case(ArtifactKind::UserStory)]
    #[case(ArtifactKind::Requirements)]
    #[case(ArtifactKind::Design)]
    #[case(ArtifactKind::Adr)]
    #[case(ArtifactKind::Task)]
    /// Covers: REQ-001 FR-002 — each Markdown type has its canonical type value.
    fn accepts_canonical_type_values(#[case] kind: ArtifactKind) {
        let diagnostics = validate_artifact(&valid_snapshot(kind, "approved"));

        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("TYPE_VALUE"))
        );
    }

    /// Covers: REQ-001 FR-008 — superseded is an allowed lifecycle value.
    #[test]
    fn accepts_superseded_lifecycle_status() {
        let diagnostics = validate_artifact(&valid_snapshot(ArtifactKind::UserStory, "superseded"));

        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("STATUS_VALUE"))
        );
    }

    /// Covers: REQ-001 FR-005 — every applicable violation is retained separately.
    #[test]
    fn retains_multiple_violations_for_one_artifact() {
        let path = fixture_path("specs/feature/user-story.md".to_string());
        let snapshot = ArtifactSnapshot::new(
            path,
            ArtifactKind::UserStory,
            Metadata::new(),
            DocumentSnapshot::empty(),
        );
        let diagnostics = validate_artifact(&snapshot);

        assert!(diagnostics.len() > 4, "missing structural violations were not aggregated");
        assert!(diagnostics.windows(2).all(|pair| {
            pair.first()
                .zip(pair.get(1))
                .is_some_and(|(left, right)| left.rule_id() <= right.rule_id())
        }));
    }

    /// Covers: REQ-001 FR-003 — unresolved values and template markers are reported.
    #[test]
    fn reports_unresolved_and_template_markers() {
        let mut metadata = Metadata::new();
        metadata.insert_scalar("id", "US-001");
        metadata.insert_scalar("owner", "TBD");
        metadata.insert_scalar("title", "{{title}}");
        let path = fixture_path("specs/feature/user-story.md".to_string());
        let snapshot = ArtifactSnapshot::new(
            path,
            ArtifactKind::UserStory,
            metadata,
            DocumentSnapshot::new(vec![SourceLine::new(3, "## TBD")], Vec::new(), Vec::new(), None),
        );
        let diagnostics = validate_artifact(&snapshot);

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic
                .rule_id()
                .as_str()
                .ends_with("PLACEHOLDER_UNRESOLVED"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("PLACEHOLDER_TEMPLATE"))
        );
    }

    /// Covers: REQ-001 FR-007 — report ordering is independent of discovery ordering.
    #[test]
    fn orders_types_then_identifiers_then_paths() {
        let mut results = Vec::new();
        for kind in ArtifactKind::ALL {
            if kind == ArtifactKind::Gherkin {
                continue;
            }
            results.push(ArtifactResult::from_snapshot(&valid_snapshot(kind, "approved")));
        }
        results.reverse();
        let report = ValidationReport::from_results(results);
        let kinds: Vec<ArtifactKind> =
            report.artifacts().iter().filter_map(ArtifactResult::kind).collect();

        assert_eq!(
            kinds,
            vec![
                ArtifactKind::Prd,
                ArtifactKind::Epic,
                ArtifactKind::UserStory,
                ArtifactKind::Requirements,
                ArtifactKind::Design,
                ArtifactKind::Adr,
                ArtifactKind::Task,
            ]
        );
    }

    /// Covers: REQ-001 FR-008 and FR-009 — warning findings fail without changing severity.
    #[test]
    fn warning_finding_fails_artifact_and_report() {
        let path = fixture_path("specs/feature/user-story.md".to_string());
        let warning = Diagnostic::new(
            path.clone(),
            None,
            "ARTIFACT.USER-STORY.TEST_WARNING",
            Severity::Warning,
            "warning finding",
            "resolve the warning",
        );
        let snapshot = valid_snapshot(ArtifactKind::UserStory, "approved")
            .with_parser_diagnostics(vec![warning]);
        let result = ArtifactResult::from_snapshot(&snapshot);
        let report = ValidationReport::from_results(vec![result.clone()]);

        assert_eq!(result.status(), ArtifactStatus::Diagnostic);
        assert_eq!(result.violations().first().map(Diagnostic::severity), Some(Severity::Warning));
        assert_eq!(report.status(), OverallStatus::Failure);
    }

    /// Covers: REQ-001 FR-002 and FR-005 — invalid metadata shapes and local references are distinct findings.
    #[test]
    fn reports_metadata_shapes_and_reference_prefixes() {
        let mut metadata = Metadata::new();
        metadata.insert("id", MetadataValue::Mapping);
        metadata.insert("type", MetadataValue::sequence(["wrong"]));
        metadata.insert("status", MetadataValue::Null);
        metadata.insert_scalar("parent", "EPIC-001");
        metadata.insert_scalar("epic", "US-001");
        metadata.insert_scalar("related", "not-an-id");
        let Some(path) = ArtifactPath::try_new("specs/unknown.md").ok() else { return };
        let snapshot = ArtifactSnapshot::new(
            path,
            ArtifactKind::UserStory,
            metadata,
            DocumentSnapshot::new(
                vec![SourceLine::new(4, "<placeholder> YYYY-MM-DD NNN")],
                vec![Heading::new(2, "Invest Check", 2)],
                Vec::new(),
                None,
            ),
        );
        let diagnostics = validate_artifact(&snapshot);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("METADATA_SHAPE"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("REFERENCE_PARENT"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("REFERENCE_SYNTAX"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("PLACEHOLDER_TEMPLATE"))
        );
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_id().as_str().starts_with("ARTIFACT.UNKNOWN"))
        );
    }

    /// Covers: REQ-001 FR-002 — a present checklist heading must contain a checklist item in its section.
    #[test]
    fn reports_empty_required_checklist_section() {
        let mut metadata = Metadata::new();
        metadata.insert_scalar("id", "TASK-001");
        let Some(path) = ArtifactPath::try_new("specs/feature/tasks.md").ok() else { return };
        let snapshot = ArtifactSnapshot::new(
            path,
            ArtifactKind::Task,
            metadata,
            DocumentSnapshot::new(
                Vec::new(),
                vec![Heading::new(2, "Definition Of Done", 1)],
                Vec::new(),
                None,
            ),
        );
        let diagnostics = validate_artifact(&snapshot);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("CHECKLIST_ITEM"))
        );
    }

    /// Covers: REQ-001 FR-002 and FR-006 — missing Gherkin structure reports each required boundary.
    #[test]
    fn reports_missing_gherkin_structure() {
        let Some(path) = ArtifactPath::try_new("specs/feature/scenarios.feature").ok() else {
            return;
        };
        let feature = crate::FeatureSnapshot::new(None, None, None, 0, false, false, false);
        let document = DocumentSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Some(feature));
        let snapshot =
            ArtifactSnapshot::new(path, ArtifactKind::Gherkin, Metadata::new(), document);
        let diagnostics = validate_artifact(&snapshot);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("HEADER_PARENT"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("HEADER_STATUS"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("FEATURE_REQUIRED"))
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("SCENARIO_REQUIRED"))
        );
        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_id().as_str().ends_with("STEP_REQUIRED"))
                .count(),
            3
        );
    }

    proptest! {
        /// Covers: REQ-001 FR-007 and FR-009 — stable report ordering is deterministic.
        #[test]
        fn report_order_is_deterministic(input in prop::collection::vec(0usize..8, 0..24)) {
            let original: Vec<ArtifactResult> = input.iter().map(|value| {
                let kind = ArtifactKind::ALL.get(*value).copied().unwrap_or(ArtifactKind::Task);
                ArtifactResult::from_snapshot(&valid_snapshot(kind, "approved"))
            }).collect();
            let mut reversed = original.clone();
            reversed.reverse();

            prop_assert_eq!(ValidationReport::from_results(original), ValidationReport::from_results(reversed));
        }
    }
}
