//! Contract tests for relationship target orchestration.

use application::{
    ArtifactCandidate, ArtifactIdentitySource, RelationshipValidator, ValidationError,
};
use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Metadata,
    MetadataValue, Severity,
};

#[derive(Clone, Debug)]
struct InMemoryRelationshipSource {
    candidates: Vec<ArtifactCandidate>,
    error: Option<String>,
}

impl InMemoryRelationshipSource {
    fn new(candidates: Vec<ArtifactCandidate>) -> Self {
        Self { candidates, error: None }
    }

    fn failing(message: &str) -> Self {
        Self { candidates: Vec::new(), error: Some(message.to_owned()) }
    }
}

impl ArtifactIdentitySource for InMemoryRelationshipSource {
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

fn result_for<'a>(
    report: &'a domain::ValidationReport,
    path: &str,
) -> Option<&'a domain::ArtifactResult> {
    report.artifacts().iter().find(|artifact| artifact.path().as_str() == path)
}

fn relationship_diagnostics(result: &domain::ArtifactResult) -> Vec<&Diagnostic> {
    result
        .violations()
        .iter()
        .filter(|diagnostic| diagnostic.rule_id().as_str().starts_with("ARTIFACT.RELATIONSHIP."))
        .collect()
}

/// Covers: REQ-003 FR-001, FR-008, and FR-010 — empty relationship validation succeeds deterministically.
#[test]
fn returns_success_for_empty_relationship_source() {
    let result = RelationshipValidator::new(InMemoryRelationshipSource::new(Vec::new())).validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert!(report.artifacts().is_empty());
    assert_eq!(report.status(), domain::OverallStatus::Success);
}

/// Covers: REQ-003 FR-004, FR-006, and FR-007 — relationship findings do not suppress unrelated results.
#[test]
fn merges_missing_relationship_findings_and_retains_unrelated_results() {
    let invalid = snapshot(
        "specs/current/story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["REQ-404", "REQ-405"]))],
    );
    let unrelated =
        snapshot("specs/current/requirements.md", ArtifactKind::Requirements, "REQ-001", &[]);
    let source = InMemoryRelationshipSource::new(vec![
        ArtifactCandidate::from_snapshot(invalid),
        ArtifactCandidate::from_snapshot(unrelated),
    ]);

    let result = RelationshipValidator::new(source).validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert_eq!(report.status(), domain::OverallStatus::Failure);
    let Some(invalid_result) = result_for(&report, "specs/current/story.md") else { return };
    assert_eq!(relationship_diagnostics(invalid_result).len(), 2);
    let Some(unrelated_result) = result_for(&report, "specs/current/requirements.md") else {
        return;
    };
    assert!(relationship_diagnostics(unrelated_result).is_empty());
}

/// Covers: REQ-003 FR-005 — wrong-kind findings retain expected and actual kind context.
#[test]
fn reports_wrong_kind_relationship_on_referencing_artifact() {
    let source = snapshot(
        "specs/current/story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("parent", MetadataValue::scalar("ADR-001"))],
    );
    let target = snapshot("specs/adr/ADR-001.md", ArtifactKind::Adr, "ADR-001", &[]);
    let source = InMemoryRelationshipSource::new(vec![
        ArtifactCandidate::from_snapshot(source),
        ArtifactCandidate::from_snapshot(target),
    ]);

    let result = RelationshipValidator::new(source).validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    let Some(artifact) = result_for(&report, "specs/current/story.md") else { return };
    let diagnostics = relationship_diagnostics(artifact);
    let Some(diagnostic) = diagnostics.first() else { return };
    assert_eq!(diagnostic.rule_id().as_str(), "ARTIFACT.RELATIONSHIP.WRONG_KIND");
    assert!(diagnostic.message().contains("PRD"));
    assert!(diagnostic.message().contains("ADR"));
}

/// Covers: REQ-003 FR-009 — structural findings remain without relationship duplicates.
#[test]
fn preserves_structural_findings_without_target_duplicates() {
    let Ok(path) = ArtifactPath::try_new("specs/current/story.md") else { return };
    let structural = Diagnostic::new(
        path.clone(),
        None,
        "ARTIFACT.USER-STORY.REFERENCE_SYNTAX",
        Severity::Error,
        "relationship value is malformed",
        "repair the relationship value",
    );
    let snapshot = ArtifactSnapshot::new(
        path,
        ArtifactKind::UserStory,
        Metadata::new(),
        DocumentSnapshot::empty(),
    )
    .with_parser_diagnostics(vec![structural]);

    let result = RelationshipValidator::new(InMemoryRelationshipSource::new(vec![
        ArtifactCandidate::from_snapshot(snapshot),
    ]))
    .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    let Some(artifact) = result_for(&report, "specs/current/story.md") else { return };
    assert!(artifact.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.USER-STORY.REFERENCE_SYNTAX"
    }));
    assert!(relationship_diagnostics(artifact).is_empty());
}

/// Covers: REQ-003 FR-007 — per-file source diagnostics remain represented in the relationship report.
#[test]
fn retains_per_file_source_diagnostics() {
    let Ok(path) = ArtifactPath::try_new("specs/current/unreadable.md") else { return };
    let source_diagnostic = Diagnostic::new(
        path.clone(),
        None,
        "ARTIFACT.USER-STORY.SOURCE_READ",
        Severity::Error,
        "artifact could not be read",
        "make the artifact readable",
    );

    let result = RelationshipValidator::new(InMemoryRelationshipSource::new(vec![
        ArtifactCandidate::from_diagnostic(path, source_diagnostic),
    ]))
    .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    let Some(artifact) = result_for(&report, "specs/current/unreadable.md") else { return };
    assert!(
        artifact.violations().iter().any(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.USER-STORY.SOURCE_READ"
        })
    );
    assert!(relationship_diagnostics(artifact).is_empty());
}

/// Covers: REQ-003 FR-010 — repository-level discovery failures remain typed and terminal.
#[test]
fn returns_typed_error_for_relationship_discovery_failure() {
    let result = RelationshipValidator::new(InMemoryRelationshipSource::failing(
        "repository root unavailable",
    ))
    .validate();

    assert!(matches!(
        result,
        Err(ValidationError::Discovery(message)) if message == "repository root unavailable"
    ));
}
