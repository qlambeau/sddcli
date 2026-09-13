//! Integration coverage for the application Spec-Ready use case.

use std::cell::Cell;
use std::rc::Rc;

use application::{
    ArtifactCandidate, ArtifactIdentitySource, EvaluateSpecReadyCommand, ReadinessError,
    SpecReadyEvaluator, ValidationError,
};
use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot,
    ImplementationPacketRef, Metadata, Severity,
};

#[derive(Clone)]
struct CountingSource {
    candidates: Vec<ArtifactCandidate>,
    calls: Rc<Cell<usize>>,
}

impl ArtifactIdentitySource for CountingSource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.candidates.clone())
    }
}

struct FailingSource;

impl ArtifactIdentitySource for FailingSource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        Err(ValidationError::Discovery("repository unavailable".to_owned()))
    }
}

fn packet() -> ImplementationPacketRef {
    match ImplementationPacketRef::try_new("specs/008-evaluate-spec-ready-status") {
        Ok(packet) => packet,
        Err(_) => std::process::abort(),
    }
}

fn candidate(path: &str) -> ArtifactCandidate {
    let Ok(path) = ArtifactPath::try_new(path) else { std::process::abort() };
    ArtifactCandidate::from_snapshot(ArtifactSnapshot::new(
        path,
        ArtifactKind::UserStory,
        Metadata::new(),
        DocumentSnapshot::empty(),
    ))
}

#[test]
fn missing_root_is_a_typed_operational_error() {
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource { candidates: Vec::new(), calls: calls.clone() };
    let evaluator = SpecReadyEvaluator::new(source);

    let result = evaluator.evaluate(&EvaluateSpecReadyCommand::new(packet()));

    assert_eq!(
        result,
        Err(ReadinessError::PacketNotFound(
            match ArtifactPath::try_new("specs/008-evaluate-spec-ready-status") {
                Ok(path) => path,
                Err(_) => std::process::abort(),
            },
        ))
    );
    assert_eq!(calls.get(), 1);
}

#[test]
fn discovery_failures_remain_typed_operational_errors() {
    let evaluator = SpecReadyEvaluator::new(FailingSource);

    let result = evaluator.evaluate(&EvaluateSpecReadyCommand::new(packet()));

    assert_eq!(
        result,
        Err(ReadinessError::Validation(ValidationError::Discovery(
            "repository unavailable".to_owned(),
        )))
    );
}

#[test]
fn source_diagnostics_are_preserved_in_a_valid_not_ready_result() {
    let path = ArtifactPath::try_new("specs/008-evaluate-spec-ready-status/user-story.md")
        .unwrap_or_else(|_| std::process::abort());
    let diagnostic = Diagnostic::new(
        path.clone(),
        None,
        "ARTIFACT.USER-STORY.SOURCE_READ",
        Severity::Error,
        "source could not be read",
        "make the source readable",
    );
    let source = CountingSource {
        candidates: vec![ArtifactCandidate::from_diagnostic(path, diagnostic)],
        calls: Rc::new(Cell::new(0)),
    };
    let evaluator = SpecReadyEvaluator::new(source);

    let Ok(result) = evaluator.evaluate(&EvaluateSpecReadyCommand::new(packet())) else {
        return;
    };

    assert!(
        result
            .result()
            .diagnostics()
            .iter()
            .any(|finding| { finding.rule_id().as_str() == "ARTIFACT.USER-STORY.SOURCE_READ" })
    );
}

#[test]
fn valid_root_returns_a_readiness_result_without_writes() {
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource {
        candidates: vec![candidate("specs/008-evaluate-spec-ready-status/user-story.md")],
        calls: calls.clone(),
    };
    let evaluator = SpecReadyEvaluator::new(source);

    let Ok(result) = evaluator.evaluate(&EvaluateSpecReadyCommand::new(packet())) else {
        return;
    };

    assert!(!result.is_ready());
    assert_eq!(calls.get(), 1);
}
