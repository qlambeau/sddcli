//! Contract tests for reciprocal relationship orchestration.

use application::{
    ArtifactCandidate, ArtifactIdentitySource, ReciprocalRelationshipValidator, ValidationError,
};
use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Heading, Metadata,
    MetadataValue, Severity,
};

#[derive(Clone, Debug)]
struct InMemoryReciprocalSource {
    candidates: Vec<ArtifactCandidate>,
    error: Option<String>,
}

impl InMemoryReciprocalSource {
    fn new(candidates: Vec<ArtifactCandidate>) -> Self {
        Self { candidates, error: None }
    }

    fn failing(message: &str) -> Self {
        Self { candidates: Vec::new(), error: Some(message.to_owned()) }
    }
}

impl ArtifactIdentitySource for InMemoryReciprocalSource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        if let Some(message) = &self.error {
            return Err(ValidationError::Discovery(message.clone()));
        }
        Ok(self.candidates.clone())
    }
}

fn snapshot(
    path: &str,
    kind: ArtifactKind,
    id: &str,
    relationships: &[(&str, MetadataValue)],
) -> ArtifactSnapshot {
    let Ok(path) = ArtifactPath::try_new(path) else { std::process::abort() };
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    for (field, value) in relationships {
        metadata.insert(*field, value.clone());
    }
    ArtifactSnapshot::new(path, kind, metadata, DocumentSnapshot::empty())
}

fn sequence(values: &[&str]) -> MetadataValue {
    MetadataValue::sequence(values.iter().copied())
}

fn valid_adr(path: &str, id: &str, related: &[&str]) -> ArtifactSnapshot {
    let Ok(path) = ArtifactPath::try_new(path) else { std::process::abort() };
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("title", "Reciprocal fixture");
    metadata.insert_scalar("type", "architecture-decision-record");
    metadata.insert_scalar("status", "approved");
    metadata.insert_scalar("created", "2026-09-05");
    metadata.insert_scalar("updated", "2026-09-05");
    metadata.insert_scalar("owner", "test-owner");
    metadata.insert("supersedes", MetadataValue::Null);
    metadata.insert("superseded_by", MetadataValue::Null);
    metadata.insert_sequence("related", related.iter().copied());
    let headings =
        ["Context", "Decision", "Alternatives Considered", "Consequences", "Follow-Up Actions"]
            .iter()
            .enumerate()
            .map(|(line, text)| Heading::new(2, *text, line + 1))
            .collect();
    ArtifactSnapshot::new(
        path,
        ArtifactKind::Adr,
        metadata,
        DocumentSnapshot::new(Vec::new(), headings, Vec::new(), None),
    )
}

fn result_for<'a>(
    report: &'a domain::ValidationReport,
    path: &str,
) -> Option<&'a domain::ArtifactResult> {
    report.artifacts().iter().find(|artifact| artifact.path().as_str() == path)
}

fn reciprocal_diagnostics(result: &domain::ArtifactResult) -> Vec<&Diagnostic> {
    result
        .violations()
        .iter()
        .filter(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.NON_RECIPROCAL"
        })
        .collect()
}

/// Covers: REQ-004 FR-001, FR-003, and FR-009 — empty and reciprocal sources return success.
#[test]
fn returns_success_for_empty_and_reciprocal_sources() {
    let source = valid_adr("specs/adr/ADR-001.md", "ADR-001", &["ADR-002"]);
    let target = valid_adr("specs/archive/001-old/ADR-002.md", "ADR-002", &["ADR-001"]);

    let empty =
        ReciprocalRelationshipValidator::new(InMemoryReciprocalSource::new(Vec::new())).validate();
    let reciprocal = ReciprocalRelationshipValidator::new(InMemoryReciprocalSource::new(vec![
        ArtifactCandidate::from_snapshot(source),
        ArtifactCandidate::from_snapshot(target),
    ]))
    .validate();

    assert!(empty.is_ok());
    assert!(reciprocal.is_ok());
    let Ok(empty) = empty else { return };
    let Ok(reciprocal) = reciprocal else { return };
    assert_eq!(empty.status(), domain::OverallStatus::Success);
    assert_eq!(reciprocal.status(), domain::OverallStatus::Success);
    assert_eq!(reciprocal.artifacts().len(), 2);
}

/// Covers: REQ-004 FR-004, FR-006, and FR-007 — target and reciprocal findings are complete and source-owned.
#[test]
fn merges_target_and_reciprocity_findings_without_suppressing_results() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("related", sequence(&["REQ-404", "US-002"]))],
    );
    let target =
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-002", &[]);
    let unrelated =
        snapshot("specs/current/requirements.md", ArtifactKind::Requirements, "REQ-001", &[]);
    let result = ReciprocalRelationshipValidator::new(InMemoryReciprocalSource::new(vec![
        ArtifactCandidate::from_snapshot(source),
        ArtifactCandidate::from_snapshot(target),
        ArtifactCandidate::from_snapshot(unrelated),
    ]))
    .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert_eq!(report.status(), domain::OverallStatus::Failure);
    let Some(source_result) = result_for(&report, "specs/current/user-story.md") else { return };
    assert_eq!(reciprocal_diagnostics(source_result).len(), 1);
    assert!(source_result.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.MISSING_TARGET"
            && diagnostic.message().contains("REQ-404")
    }));
    assert!(result_for(&report, "specs/current/requirements.md").is_some());
}

/// Covers: REQ-004 FR-008 — source and structural diagnostics are retained without reciprocity duplicates.
#[test]
fn retains_source_diagnostics_and_skips_invalid_values() {
    let Ok(path) = ArtifactPath::try_new("specs/current/user-story.md") else { return };
    let source_diagnostic = Diagnostic::new(
        path.clone(),
        None,
        "ARTIFACT.USER-STORY.SOURCE_READ",
        Severity::Error,
        "artifact could not be read",
        "make the artifact readable",
    );
    let snapshot = ArtifactSnapshot::new(
        path,
        ArtifactKind::UserStory,
        Metadata::new(),
        DocumentSnapshot::empty(),
    )
    .with_parser_diagnostics(vec![source_diagnostic]);

    let result = ReciprocalRelationshipValidator::new(InMemoryReciprocalSource::new(vec![
        ArtifactCandidate::from_snapshot(snapshot),
    ]))
    .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    let Some(artifact) = result_for(&report, "specs/current/user-story.md") else { return };
    assert!(
        artifact.violations().iter().any(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.USER-STORY.SOURCE_READ"
        })
    );
    assert!(reciprocal_diagnostics(artifact).is_empty());
}

/// Covers: REQ-004 FR-005 and FR-007 — both supersession mappings remain independently diagnosable.
#[test]
fn reports_both_supersession_directions_on_their_sources() {
    let superseder = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("supersedes", MetadataValue::scalar("US-002"))],
    );
    let superseded_by = snapshot(
        "specs/current/second-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("superseded_by", MetadataValue::scalar("US-004"))],
    );
    let target =
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-002", &[]);
    let successor =
        snapshot("specs/archive/002-old/user-story.md", ArtifactKind::UserStory, "US-004", &[]);
    let result = ReciprocalRelationshipValidator::new(InMemoryReciprocalSource::new(vec![
        ArtifactCandidate::from_snapshot(superseder),
        ArtifactCandidate::from_snapshot(superseded_by),
        ArtifactCandidate::from_snapshot(target),
        ArtifactCandidate::from_snapshot(successor),
    ]))
    .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert_eq!(
        report
            .artifacts()
            .iter()
            .flat_map(domain::ArtifactResult::violations)
            .filter(|diagnostic| {
                diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.NON_RECIPROCAL"
            })
            .count(),
        2
    );
}

/// Covers: REQ-004 FR-010 — repeated application validation returns the same ordered report.
#[test]
fn returns_identical_reports_for_repeated_validation() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("related", sequence(&["US-002"]))],
    );
    let target =
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-002", &[]);
    let source = InMemoryReciprocalSource::new(vec![
        ArtifactCandidate::from_snapshot(source),
        ArtifactCandidate::from_snapshot(target),
    ]);

    let first = ReciprocalRelationshipValidator::new(source.clone()).validate();
    let second = ReciprocalRelationshipValidator::new(source).validate();

    assert_eq!(first, second);
}

/// Covers: REQ-004 FR-010 — repository discovery failure remains typed and terminal.
#[test]
fn returns_typed_error_for_discovery_failure() {
    let result = ReciprocalRelationshipValidator::new(InMemoryReciprocalSource::failing(
        "repository root unavailable",
    ))
    .validate();

    assert!(matches!(
        result,
        Err(ValidationError::Discovery(message)) if message == "repository root unavailable"
    ));
}
