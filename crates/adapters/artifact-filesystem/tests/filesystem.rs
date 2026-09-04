//! Contract tests for canonical filesystem discovery and parser boundaries.

use std::fs;

use application::{ArtifactSource, Validator};
use artifact_filesystem::{FilesystemArtifactSource, parse_artifact};
use domain::{ArtifactKind, ArtifactPath, ArtifactStatus, OverallStatus};

const MARKDOWN_HEADINGS: &str = "# User Story\n\n## Story Card\n";

fn write_file(root: &std::path::Path, relative: &str, contents: &str) -> bool {
    let path = root.join(relative);
    let Some(parent) = path.parent() else { return false };
    fs::create_dir_all(parent).is_ok() && fs::write(path, contents).is_ok()
}

/// Covers: REQ-001 FR-001 — canonical active paths are discovered while templates and archives are excluded.
#[test]
fn discovers_only_canonical_active_paths() {
    let Ok(directory) = tempfile::tempdir() else { return };
    let files = [
        ("specs/prds/PRD-001.md", "---\nid: PRD-001\n---\n"),
        ("specs/prds/PRD-001-epics/EPIC-001.md", "---\nid: EPIC-001\n---\n"),
        ("specs/feature/user-story.md", "---\nid: US-001\n---\n"),
        ("specs/feature/scenarios.feature", "Feature: Feature\n"),
        ("specs/feature/requirements.md", "---\nid: REQ-001\n---\n"),
        ("specs/feature/design.md", "---\nid: DES-001\n---\n"),
        ("specs/adr/ADR-001.md", "---\nid: ADR-001\n---\n"),
        ("specs/feature/tasks.md", "---\nid: TASK-001\n---\n"),
        ("specs/templates/feature/user-story.md", "template"),
        ("specs/archive/001-old/user-story.md", "archive"),
        ("specs/feature/supporting.md", "supporting"),
    ];
    assert!(files.iter().all(|(path, contents)| write_file(directory.path(), path, contents)));

    let source = FilesystemArtifactSource::new(directory.path());
    let result = source.discover();

    assert!(result.is_ok());
    let Ok(candidates) = result else { return };
    assert_eq!(candidates.len(), 8);
    assert!(candidates.iter().all(|candidate| candidate.path().as_str().starts_with("specs/")));
    assert!(candidates.iter().all(|candidate| !candidate.path().as_str().contains("templates")));
    assert!(candidates.iter().all(|candidate| !candidate.path().as_str().contains("archive")));
}

/// Covers: REQ-001 FR-002 and FR-009 — frontmatter and Markdown structure are normalized at the boundary.
#[test]
fn parses_yaml_frontmatter_and_markdown_structure() {
    let Ok(path) = ArtifactPath::try_new("specs/feature/user-story.md") else { return };
    let contents = "---\nid: US-001\ntitle: Validate\nstatus: approved\nrelated: []\n---\n\n";
    let snapshot =
        parse_artifact(path, ArtifactKind::UserStory, &format!("{contents}{MARKDOWN_HEADINGS}"));

    assert!(
        matches!(snapshot.metadata().get("id"), Some(domain::MetadataValue::Scalar(value)) if value == "US-001")
    );
    assert!(snapshot.document().headings().iter().any(|heading| heading.text() == "User Story"));
    assert!(snapshot.parser_diagnostics().is_empty());
}

/// Covers: REQ-001 FR-002 — Markdown checklist items are retained with their source location.
#[test]
fn parses_markdown_checklist_items() {
    let Ok(path) = ArtifactPath::try_new("specs/feature/tasks.md") else { return };
    let snapshot =
        parse_artifact(path, ArtifactKind::Task, "## Definition Of Done\n- [x] complete\n");

    assert_eq!(snapshot.document().checklist_items().len(), 1);
}

/// Covers: REQ-001 FR-006 — malformed frontmatter remains a path-based diagnostic candidate.
#[test]
fn reports_malformed_frontmatter_without_panicking() {
    let Ok(path) = ArtifactPath::try_new("specs/feature/user-story.md") else { return };
    let snapshot = parse_artifact(path, ArtifactKind::UserStory, "---\nid: [\n---\n# User Story\n");

    assert!(!snapshot.parser_diagnostics().is_empty());
    assert_eq!(snapshot.path().as_str(), "specs/feature/user-story.md");
}

/// Covers: REQ-001 FR-002 — Gherkin headers, scenarios, and step types are normalized.
#[test]
fn parses_gherkin_feature_and_steps() {
    let Ok(path) = ArtifactPath::try_new("specs/feature/scenarios.feature") else { return };
    let contents = "# parent: US-001\n# status: approved\n\nFeature: Validate\n\n  Scenario: Works\n    Given an active repository\n    When validation runs\n    Then the report succeeds\n";
    let snapshot = parse_artifact(path, ArtifactKind::Gherkin, contents);
    let feature = snapshot.document().feature();

    assert!(feature.is_some());
    let Some(feature) = feature else { return };
    assert_eq!(feature.parent(), Some("US-001"));
    assert_eq!(feature.status(), Some("approved"));
    assert_eq!(feature.scenario_count(), 1);
    assert!(feature.has_given() && feature.has_when() && feature.has_then());
    assert!(snapshot.parser_diagnostics().is_empty());
}

/// Covers: REQ-001 FR-006 — malformed Gherkin becomes a source-located validation finding.
#[test]
fn reports_malformed_gherkin() {
    let Ok(path) = ArtifactPath::try_new("specs/feature/scenarios.feature") else { return };
    let snapshot = parse_artifact(path, ArtifactKind::Gherkin, "not a Gherkin feature");

    assert!(
        snapshot
            .parser_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("GHERKIN_PARSE"))
    );
}

/// Covers: REQ-001 FR-006 — unreadable canonical files remain diagnostic candidates.
#[test]
fn keeps_unreadable_canonical_path_as_diagnostic() {
    let Ok(directory) = tempfile::tempdir() else { return };
    let path = directory.path().join("specs/feature/user-story.md");
    assert!(fs::create_dir_all(&path).is_ok());

    let result = FilesystemArtifactSource::new(directory.path()).discover();

    assert!(result.is_ok());
    let Ok(candidates) = result else { return };
    assert_eq!(candidates.len(), 1);
    let Some(candidate) = candidates.first() else { return };
    assert_eq!(candidate.path().as_str(), "specs/feature/user-story.md");
    assert_eq!(candidate.snapshot(), None);
    assert!(candidate.diagnostic().is_some());
}

/// Covers: REQ-001 FR-004 — an empty canonical tree is a successful source result.
#[test]
fn discovers_empty_active_set() {
    let Ok(directory) = tempfile::tempdir() else { return };

    let result = FilesystemArtifactSource::new(directory.path()).discover();

    assert!(result.is_ok());
    let Ok(candidates) = result else { return };
    assert!(candidates.is_empty());
}

/// Covers: REQ-001 FR-005 — parsed invalid artifacts remain available for complete validation.
#[test]
fn parsed_candidate_can_be_validated_after_discovery() {
    let Ok(directory) = tempfile::tempdir() else { return };
    assert!(write_file(directory.path(), "specs/feature/user-story.md", "---\n---\n"));
    let source = FilesystemArtifactSource::new(directory.path());
    let result = source.discover();

    assert!(result.is_ok());
    let Ok(candidates) = result else { return };
    let Some(candidate) = candidates.first() else { return };
    let Some(snapshot) = candidate.snapshot() else { return };
    let artifact = domain::ArtifactResult::from_snapshot(snapshot);
    assert_eq!(artifact.status(), ArtifactStatus::Diagnostic);
}

fn valid_artifact_contents(kind: ArtifactKind) -> String {
    if kind == ArtifactKind::Gherkin {
        return "# parent: US-001\n# status: approved\n\nFeature: Validate\n\n  Scenario: Works\n    Given an active repository\n    When validation runs\n    Then the report succeeds\n".to_string();
    }
    let id = format!("{}-001", kind.id_prefix().unwrap_or("NO"));
    let type_value = kind.expected_type().unwrap_or("unknown");
    let parent = match kind {
        ArtifactKind::Prd | ArtifactKind::Adr => "null",
        ArtifactKind::Epic | ArtifactKind::UserStory => "PRD-001",
        _ => "US-001",
    };
    let additional = match kind {
        ArtifactKind::Prd => "scope: project\n",
        ArtifactKind::UserStory => "epic: EPIC-001\nfeature: validate-artifacts\n",
        _ => "",
    };
    let references = match kind {
        ArtifactKind::Prd | ArtifactKind::Adr => "supersedes: null\nsuperseded_by: null\n",
        _ => "depends_on: []\nrequires: []\nblockers: []\n",
    };
    let headings = headings_for(kind);
    let mut document = String::new();
    for heading in headings {
        document.push_str("## ");
        document.push_str(heading);
        document.push('\n');
    }
    format!(
        "---\nid: {id}\ntitle: Concrete artifact\ntype: {type_value}\nstatus: approved\ncreated: 2026-09-04\nupdated: 2026-09-04\nowner: project-owner\nparent: {parent}\n{additional}{references}related: []\n---\n\n{document}- [x] complete\n"
    )
}

fn headings_for(kind: ArtifactKind) -> &'static [&'static str] {
    match kind {
        ArtifactKind::Prd => &[
            "Product Requirements Document",
            "Vision And Problem",
            "Target Personas And Journeys",
            "Success Metrics",
            "Functional Scope And Epics",
            "Non-Functional Requirements",
            "Assumptions And Out Of Scope",
            "Open Questions",
            "Decision Log",
            "Review Checklist",
        ],
        ArtifactKind::Epic => &[
            "Epic Brief",
            "Outcome Statement",
            "Capability Boundaries",
            "Candidate Vertical Slices",
            "Success Criteria",
            "Dependencies",
            "Open Questions",
            "Readiness Checklist",
        ],
        ArtifactKind::UserStory => &[
            "User Story",
            "Story Card",
            "Context And Value",
            "Business Rules",
            "Examples",
            "Acceptance Criteria",
            "Scope Boundaries",
            "Dependencies",
            "Open Questions",
            "Invest Check",
        ],
        ArtifactKind::Requirements => &[
            "Requirements",
            "Purpose And Actors",
            "Preconditions",
            "Inputs And Outputs",
            "Functional Requirements",
            "Postconditions And Invariants",
            "Edge And Failure Behavior",
            "Quality Requirements",
            "Traceability",
            "Supplementary Notes",
        ],
        ArtifactKind::Design => &[
            "Design",
            "Context And Constraints",
            "Proposed Design",
            "Components And Responsibilities",
            "Interfaces And Contracts",
            "Data And State Flow",
            "Security, Performance, And Operations",
            "Alternatives Considered",
            "Risks And Open Decisions",
            "Verification Approach",
        ],
        ArtifactKind::Adr => &[
            "Architecture Decision Record",
            "Context",
            "Decision",
            "Alternatives Considered",
            "Consequences",
            "Follow-Up Actions",
            "Supplementary Notes",
            "More Notes",
            "Final Notes",
            "Additional Notes",
        ],
        ArtifactKind::Task => &[
            "Tasks",
            "Implementation Approach",
            "Ordered Tasks",
            "Test And Verification Plan",
            "Rollout And Recovery",
            "Definition Of Done",
        ],
        ArtifactKind::Gherkin => &["Gherkin"],
    }
}

fn canonical_path(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Prd => "specs/prds/PRD-001.md",
        ArtifactKind::Epic => "specs/prds/PRD-001-epics/EPIC-001.md",
        ArtifactKind::UserStory => "specs/validate/user-story.md",
        ArtifactKind::Gherkin => "specs/validate/scenarios.feature",
        ArtifactKind::Requirements => "specs/validate/requirements.md",
        ArtifactKind::Design => "specs/validate/design.md",
        ArtifactKind::Adr => "specs/adr/ADR-001.md",
        ArtifactKind::Task => "specs/validate/tasks.md",
    }
}

/// Covers: REQ-001 FR-001 through FR-009 — all eight supported types pass through the real source and use case.
#[test]
fn validates_all_eight_types_as_one_successful_report() {
    let Ok(directory) = tempfile::tempdir() else { return };
    let kinds = ArtifactKind::ALL;
    assert!(kinds.iter().all(|kind| {
        write_file(directory.path(), canonical_path(*kind), &valid_artifact_contents(*kind))
    }));
    let source = FilesystemArtifactSource::new(directory.path());
    let result = Validator::new(source).validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert_eq!(report.status(), OverallStatus::Success, "report: {report:?}");
    assert_eq!(report.artifacts().len(), 8);
    assert!(report.artifacts().iter().all(
        |artifact| artifact.status() == ArtifactStatus::Ok && artifact.violations().is_empty()
    ));
}

/// Covers: REQ-001 FR-005 and FR-008 — multiple violations remain separate and warnings fail strictly.
#[test]
fn reports_complete_multi_violation_failure() {
    let Ok(directory) = tempfile::tempdir() else { return };
    let invalid = "---\nid: US-invalid\ntitle: TBD\ntype: wrong\nstatus: unknown\ncreated: 2026-09-04\nupdated: 2026-09-04\nowner: owner\nparent: nope\nepic: nope\nfeature: feature\ndepends_on: []\nrequires: []\nblockers: []\nrelated: []\n---\n# User Story\n";
    assert!(write_file(directory.path(), "specs/validate/user-story.md", invalid));
    let result = Validator::new(FilesystemArtifactSource::new(directory.path())).validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    let Some(artifact) = report.artifacts().first() else { return };
    assert_eq!(report.status(), OverallStatus::Failure);
    assert_eq!(artifact.status(), ArtifactStatus::Diagnostic);
    assert!(artifact.violations().len() > 5);
    assert!(artifact.violations().iter().all(|diagnostic| {
        diagnostic.path().as_str() == "specs/validate/user-story.md"
            && !diagnostic.rule_id().as_str().is_empty()
            && !diagnostic.message().is_empty()
            && !diagnostic.remediation().is_empty()
    }));
}

/// Covers: REQ-001 FR-009 — repeated validation is identical and leaves source bytes unchanged.
#[test]
fn validation_is_deterministic_and_read_only() {
    let Ok(directory) = tempfile::tempdir() else { return };
    let relative = "specs/validate/user-story.md";
    let contents = valid_artifact_contents(ArtifactKind::UserStory);
    assert!(write_file(directory.path(), relative, &contents));
    let path = directory.path().join(relative);
    let Ok(before) = fs::read(&path) else { return };
    let source = FilesystemArtifactSource::new(directory.path());

    let first = Validator::new(source.clone()).validate();
    let second = Validator::new(source).validate();

    assert_eq!(first, second);
    let Ok(after) = fs::read(path) else { return };
    assert_eq!(before, after);
}
