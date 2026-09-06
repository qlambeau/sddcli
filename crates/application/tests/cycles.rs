//! Contract tests for cycle-aware application orchestration.

use std::cell::Cell;
use std::rc::Rc;

use application::{ArtifactCandidate, ArtifactIdentitySource, CycleValidator, ValidationError};
use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Metadata,
    MetadataValue, OverallStatus, Severity,
};

#[derive(Clone, Debug)]
struct CountingSource {
    candidates: Vec<ArtifactCandidate>,
    calls: Rc<Cell<usize>>,
    error: Option<String>,
}

impl CountingSource {
    fn new(candidates: Vec<ArtifactCandidate>, calls: Rc<Cell<usize>>) -> Self {
        Self { candidates, calls, error: None }
    }

    fn failing(message: &str, calls: Rc<Cell<usize>>) -> Self {
        Self { candidates: Vec::new(), calls, error: Some(message.to_owned()) }
    }
}

impl ArtifactIdentitySource for CountingSource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        self.calls.set(self.calls.get() + 1);
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

fn cycle_diagnostics(result: &domain::ArtifactResult) -> Vec<&Diagnostic> {
    result
        .violations()
        .iter()
        .filter(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.CYCLE")
        .collect()
}

/// Covers: REQ-005 FR-001, FR-012, and FR-013 — empty cycle-aware validation succeeds.
#[test]
fn returns_success_for_empty_source() {
    let calls = Rc::new(Cell::new(0));
    let result = CycleValidator::new(CountingSource::new(Vec::new(), Rc::clone(&calls))).validate();
    assert!(result.is_ok());
    let Ok(report) = result else { return };

    assert_eq!(report.status(), OverallStatus::Success);
    assert!(report.artifacts().is_empty());
    assert_eq!(calls.get(), 1);
}

/// Covers: REQ-005 FR-002, FR-004, FR-010, and FR-012 — cycle findings merge with target findings.
#[test]
fn merges_cycle_and_existing_relationship_findings_by_source() {
    let first = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["US-002", "REQ-404"]))],
    );
    let second = snapshot(
        "specs/archive/001-old/user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("depends_on", sequence(&["US-001"]))],
    );
    let unrelated =
        snapshot("specs/current/requirements.md", ArtifactKind::Requirements, "REQ-001", &[]);
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource::new(
        vec![
            ArtifactCandidate::from_snapshot(first),
            ArtifactCandidate::from_snapshot(second),
            ArtifactCandidate::from_snapshot(unrelated),
        ],
        Rc::clone(&calls),
    );

    let result = CycleValidator::new(source).validate();
    assert!(result.is_ok());
    let Ok(report) = result else { return };

    assert_eq!(report.status(), OverallStatus::Failure);
    assert_eq!(report.artifacts().len(), 3);
    assert_eq!(calls.get(), 1);
    let Some(first_result) = result_for(&report, "specs/current/first-user-story.md") else {
        return;
    };
    assert_eq!(cycle_diagnostics(first_result).len(), 1);
    assert!(first_result.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.MISSING_TARGET"
            && diagnostic.message().contains("REQ-404")
    }));
    let Some(second_result) = result_for(&report, "specs/archive/001-old/user-story.md") else {
        return;
    };
    assert_eq!(cycle_diagnostics(second_result).len(), 1);
    let Some(unrelated_result) = result_for(&report, "specs/current/requirements.md") else {
        return;
    };
    assert!(cycle_diagnostics(unrelated_result).is_empty());
}

/// Covers: REQ-005 FR-005, FR-010, and FR-011 — non-reciprocal supersession entries remain cycle-eligible.
#[test]
fn retains_non_reciprocal_findings_with_supersession_cycles() {
    let first = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("supersedes", MetadataValue::scalar("US-002"))],
    );
    let second = snapshot(
        "specs/current/second-user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("supersedes", MetadataValue::scalar("US-003"))],
    );
    let third = snapshot(
        "specs/current/third-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("supersedes", MetadataValue::scalar("US-001"))],
    );
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource::new(
        vec![
            ArtifactCandidate::from_snapshot(first),
            ArtifactCandidate::from_snapshot(second),
            ArtifactCandidate::from_snapshot(third),
        ],
        calls,
    );

    let result = CycleValidator::new(source).validate();
    assert!(result.is_ok());
    let Ok(report) = result else { return };

    assert_eq!(
        report
            .artifacts()
            .iter()
            .flat_map(domain::ArtifactResult::violations)
            .filter(|diagnostic| { diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.CYCLE" })
            .count(),
        3
    );
    let Some(first_result) = result_for(&report, "specs/current/first-user-story.md") else {
        return;
    };
    assert!(first_result.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.NON_RECIPROCAL"
    }));
    assert_eq!(cycle_diagnostics(first_result).len(), 1);
}

/// Covers: REQ-005 FR-011 and FR-014 — per-file source diagnostics and repeated reports survive unchanged.
#[test]
fn preserves_source_diagnostics_and_returns_identical_reports() {
    let Ok(path) = ArtifactPath::try_new("specs/current/unreadable.md") else {
        return;
    };
    let source_diagnostic = Diagnostic::new(
        path.clone(),
        None,
        "ARTIFACT.USER-STORY.SOURCE_READ",
        Severity::Error,
        "artifact could not be read",
        "make the artifact readable",
    );
    let candidate = ArtifactCandidate::from_diagnostic(path, source_diagnostic);
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource::new(vec![candidate], Rc::clone(&calls));

    let first = CycleValidator::new(source.clone()).validate();
    let second = CycleValidator::new(source).validate();

    assert_eq!(first, second);
    assert_eq!(calls.get(), 2);
    let Ok(report) = first else { return };
    let Some(artifact) = report.artifacts().first() else { return };
    assert_eq!(artifact.violations().len(), 1);
    let Some(diagnostic) = artifact.violations().first() else { return };
    assert_eq!(diagnostic.rule_id().as_str(), "ARTIFACT.USER-STORY.SOURCE_READ");
}

/// Covers: REQ-005 FR-010 and FR-014 — repository discovery errors remain typed and terminal.
#[test]
fn returns_typed_error_for_discovery_failure() {
    let calls = Rc::new(Cell::new(0));
    let result = CycleValidator::new(CountingSource::failing(
        "repository root unavailable",
        Rc::clone(&calls),
    ))
    .validate();

    assert!(matches!(
        result,
        Err(ValidationError::Discovery(message)) if message == "repository root unavailable"
    ));
    assert_eq!(calls.get(), 1);
}
