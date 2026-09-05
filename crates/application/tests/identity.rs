//! Contract tests for repository-wide identity orchestration.

use application::{ArtifactCandidate, ArtifactIdentitySource, IdentityValidator, ValidationError};
use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Metadata, Severity,
};

fn snapshot(path: &str, kind: ArtifactKind, id: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    let Ok(path) = ArtifactPath::try_new(path) else { std::process::abort() };
    ArtifactSnapshot::new(path, kind, metadata, DocumentSnapshot::empty())
}

#[derive(Clone, Debug)]
struct InMemoryIdentitySource {
    candidates: Vec<ArtifactCandidate>,
    error: Option<String>,
}

impl InMemoryIdentitySource {
    fn new(candidates: Vec<ArtifactCandidate>) -> Self {
        Self { candidates, error: None }
    }

    fn failing(message: &str) -> Self {
        Self { candidates: Vec::new(), error: Some(message.to_owned()) }
    }
}

impl ArtifactIdentitySource for InMemoryIdentitySource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        if let Some(message) = &self.error {
            return Err(ValidationError::Discovery(message.clone()));
        }
        Ok(self.candidates.clone())
    }
}

fn result_for<'a>(
    report: &'a domain::ValidationReport,
    path: &str,
) -> Option<&'a domain::ArtifactResult> {
    report.artifacts().iter().find(|artifact| artifact.path().as_str() == path)
}

/// Covers: REQ-002 FR-001 and FR-005 — the identity use case returns a successful empty report.
#[test]
fn returns_success_for_empty_identity_source() {
    let result = IdentityValidator::new(InMemoryIdentitySource::new(Vec::new())).validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert!(report.artifacts().is_empty());
    assert_eq!(report.status(), domain::OverallStatus::Success);
}

/// Covers: REQ-002 FR-003 and FR-007 — duplicate findings do not suppress unrelated results.
#[test]
fn merges_identity_findings_into_all_affected_results() {
    let candidates = vec![
        ArtifactCandidate::from_snapshot(snapshot(
            "specs/current/user-story.md",
            ArtifactKind::UserStory,
            "US-001",
        )),
        ArtifactCandidate::from_snapshot(snapshot(
            "specs/archive/001-old/user-story.md",
            ArtifactKind::UserStory,
            "US-001",
        )),
        ArtifactCandidate::from_snapshot(snapshot(
            "specs/current/requirements.md",
            ArtifactKind::Requirements,
            "REQ-001",
        )),
    ];

    let report = IdentityValidator::new(InMemoryIdentitySource::new(candidates)).validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };

    assert_eq!(report.status(), domain::OverallStatus::Failure);
    for path in ["specs/current/user-story.md", "specs/archive/001-old/user-story.md"] {
        assert!(result_for(&report, path).is_some());
        let Some(result) = result_for(&report, path) else { return };
        assert!(result.violations().iter().any(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.IDENTITY.DUPLICATE_ID"
        }));
    }
    assert!(result_for(&report, "specs/current/requirements.md").is_some());
    let Some(unrelated) = result_for(&report, "specs/current/requirements.md") else { return };
    assert!(
        !unrelated
            .violations()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str().starts_with("ARTIFACT.IDENTITY.") })
    );
}

/// Covers: REQ-002 FR-006 — malformed frontmatter remains structural and is not an identity.
#[test]
fn retains_structural_diagnostics_without_reparsing_identity() {
    let Ok(path) = ArtifactPath::try_new("specs/current/user-story.md") else { return };
    let parser_diagnostic = Diagnostic::new(
        path.clone(),
        None,
        "ARTIFACT.USER-STORY.FRONTMATTER_PARSE",
        Severity::Error,
        "frontmatter is malformed",
        "repair the frontmatter",
    );
    let snapshot = ArtifactSnapshot::new(
        path,
        ArtifactKind::UserStory,
        Metadata::new(),
        DocumentSnapshot::empty(),
    )
    .with_parser_diagnostics(vec![parser_diagnostic]);

    let report = IdentityValidator::new(InMemoryIdentitySource::new(vec![
        ArtifactCandidate::from_snapshot(snapshot),
    ]))
    .validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert!(result_for(&report, "specs/current/user-story.md").is_some());
    let Some(result) = result_for(&report, "specs/current/user-story.md") else { return };

    assert!(result.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.USER-STORY.FRONTMATTER_PARSE"
    }));
    assert!(
        !result
            .violations()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str().starts_with("ARTIFACT.IDENTITY.") })
    );
}

/// Covers: REQ-002 FR-008 — repository discovery failures remain typed and terminal.
#[test]
fn returns_typed_error_for_identity_discovery_failure() {
    let result =
        IdentityValidator::new(InMemoryIdentitySource::failing("repository root unavailable"))
            .validate();

    assert!(matches!(
        result,
        Err(ValidationError::Discovery(message)) if message == "repository root unavailable"
    ));
}
