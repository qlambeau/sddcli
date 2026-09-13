//! Application integration tests for guarded completion transitions.

use std::collections::BTreeMap;

use application::{
    ArtifactCandidate, ArtifactPacketPromotionStore, ArtifactPromotionStore, BatchPromotionEntry,
    Clock, PacketPromoter, PacketPromotionOutcome, PromoteArtifactCommand, PromotePacketCommand,
    PromotionError, PromotionOutcome, SourceArtifact,
};
use application::{ArtifactIdentitySource, Promoter, ValidationError};
use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, DocumentSnapshot, FeatureSnapshot,
    ImplementationPacketRef, LifecycleState, Metadata, PromotionActor, PromotionPlan,
    PromotionTimestamp, ReleaseFeature, ReleaseSnapshot,
};
use std::cell::Cell;
use std::rc::Rc;

fn path(value: &str) -> ArtifactPath {
    match ArtifactPath::try_new(value) {
        Ok(path) => path,
        Err(_) => std::process::abort(),
    }
}

fn actor() -> PromotionActor {
    match PromotionActor::try_new("reviewer-1") {
        Ok(actor) => actor,
        Err(_) => std::process::abort(),
    }
}

struct TestClock {
    calls: Rc<Cell<usize>>,
    fail: bool,
}

impl TestClock {
    fn failing(calls: Rc<Cell<usize>>) -> Self {
        Self { calls, fail: true }
    }
}

impl Clock for TestClock {
    fn now(&self) -> Result<PromotionTimestamp, PromotionError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            Err(PromotionError::Clock("clock should not run".to_string()))
        } else {
            Ok(PromotionTimestamp::from_unix_seconds(42))
        }
    }
}

#[derive(Debug)]
struct TestSingleStore {
    source: SourceArtifact,
    commits: Rc<Cell<usize>>,
}

impl ArtifactPromotionStore for TestSingleStore {
    fn load(&self, path: &ArtifactPath) -> Result<SourceArtifact, PromotionError> {
        if self.source.path() == path {
            Ok(self.source.clone())
        } else {
            Err(PromotionError::TargetNotFound(path.clone()))
        }
    }

    fn commit(
        &mut self,
        source: &SourceArtifact,
        _plan: &PromotionPlan,
    ) -> Result<SourceArtifact, PromotionError> {
        self.commits.set(self.commits.get() + 1);
        Ok(source.clone())
    }
}

fn release(
    status: &str,
    features: Vec<ReleaseFeature>,
    evidence: bool,
    commit: bool,
) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    for (name, value) in [
        ("id", "REL-001"),
        ("title", "Release"),
        ("type", "release-record"),
        ("status", status),
        ("version", "v0.1.0"),
        ("commit", if commit { "abc123" } else { "" }),
        ("date", "2026-09-13"),
        ("owner", "reviewer"),
    ] {
        metadata.insert_scalar(name, value);
    }
    metadata.insert_sequence("related", Vec::<String>::new());
    ArtifactSnapshot::new(
        path("specs/releases/REL-001.md"),
        ArtifactKind::Release,
        metadata,
        DocumentSnapshot::empty().with_release(ReleaseSnapshot::new(features, evidence, commit)),
    )
}

/// Covers: REQ-009 FR-005, FR-009, and FR-010 — incomplete releases do not read the clock or commit.
#[test]
fn incomplete_release_is_rejected_without_clock_or_commit() {
    let release =
        release("approved", vec![ReleaseFeature::new("US-009", "implemented")], false, false);
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource {
        candidates: vec![ArtifactCandidate::from_snapshot(release.clone())],
        calls: calls.clone(),
    };
    let clock_calls = Rc::new(Cell::new(0));
    let store_commits = Rc::new(Cell::new(0));
    let store = TestSingleStore {
        source: SourceArtifact::new(release.path().clone(), "---\nstatus: approved\n---\n"),
        commits: store_commits.clone(),
    };
    let mut promoter = Promoter::new(source, store, TestClock::failing(clock_calls.clone()));

    let result = promoter.promote(&PromoteArtifactCommand::new(
        release.path().clone(),
        LifecycleState::Released,
        actor(),
    ));

    let Ok(PromotionOutcome::Rejected(diagnostics)) = result else {
        assert!(matches!(result, Ok(PromotionOutcome::Rejected(_))));
        return;
    };
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str().ends_with("RELEASE_VERIFICATION") })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str().ends_with("RELEASE_COMMIT") })
    );
    assert_eq!(store_commits.get(), 0);
    assert_eq!(clock_calls.get(), 0);
}

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

#[derive(Debug)]
struct TestPacketStore {
    sources: BTreeMap<ArtifactPath, SourceArtifact>,
    commits: Rc<Cell<usize>>,
}

impl ArtifactPacketPromotionStore for TestPacketStore {
    fn load_batch(&self, paths: &[ArtifactPath]) -> Result<Vec<SourceArtifact>, PromotionError> {
        paths
            .iter()
            .map(|path| {
                self.sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| PromotionError::TargetNotFound(path.clone()))
            })
            .collect()
    }

    fn commit_batch(
        &mut self,
        entries: &[BatchPromotionEntry],
    ) -> Result<Vec<SourceArtifact>, PromotionError> {
        self.commits.set(self.commits.get() + 1);
        Ok(entries.iter().map(|entry| entry.source().clone()).collect())
    }
}

fn archived_packet_snapshots() -> Vec<ArtifactSnapshot> {
    let packet = "specs/archive/009-guarded-completion";
    vec![
        markdown_packet_snapshot(
            &format!("{packet}/user-story.md"),
            ArtifactKind::UserStory,
            "US-009",
        ),
        gherkin_packet_snapshot(&format!("{packet}/scenarios.feature")),
        markdown_packet_snapshot(
            &format!("{packet}/requirements.md"),
            ArtifactKind::Requirements,
            "REQ-009",
        ),
        markdown_packet_snapshot(&format!("{packet}/design.md"), ArtifactKind::Design, "DES-009"),
        markdown_packet_snapshot(&format!("{packet}/tasks.md"), ArtifactKind::Task, "TASK-009"),
    ]
}

fn markdown_packet_snapshot(path_value: &str, kind: ArtifactKind, id: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("status", "implemented");
    ArtifactSnapshot::new(path(path_value), kind, metadata, DocumentSnapshot::empty())
}

fn gherkin_packet_snapshot(path_value: &str) -> ArtifactSnapshot {
    ArtifactSnapshot::new(
        path(path_value),
        ArtifactKind::Gherkin,
        Metadata::new(),
        DocumentSnapshot::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(FeatureSnapshot::new(
                Some("US-009".to_string()),
                Some("implemented".to_string()),
                Some("Completion".to_string()),
                1,
                true,
                true,
                true,
            )),
        ),
    )
}

/// Covers: REQ-009 FR-006, FR-007, and FR-009 — archive prerequisites reject without mutation.
#[test]
fn archive_prerequisite_rejection_does_not_use_clock_or_batch_commit() {
    let snapshots = archived_packet_snapshots();
    let calls = Rc::new(Cell::new(0));
    let source = CountingSource {
        candidates: snapshots.iter().cloned().map(ArtifactCandidate::from_snapshot).collect(),
        calls: calls.clone(),
    };
    let store_commits = Rc::new(Cell::new(0));
    let store = TestPacketStore {
        sources: snapshots
            .iter()
            .map(|snapshot| {
                (snapshot.path().clone(), SourceArtifact::new(snapshot.path().clone(), "source"))
            })
            .collect(),
        commits: store_commits.clone(),
    };
    let Ok(packet) = ImplementationPacketRef::try_new("specs/archive/009-guarded-completion")
    else {
        std::process::abort();
    };
    let clock_calls = Rc::new(Cell::new(0));
    let mut promoter = PacketPromoter::new(source, store, TestClock::failing(clock_calls.clone()));

    let result = promoter.promote(&PromotePacketCommand::new(packet, actor()));

    assert!(matches!(result, Ok(PacketPromotionOutcome::Rejected(_))));
    assert_eq!(calls.get(), 1);
    assert_eq!(store_commits.get(), 0);
    assert_eq!(clock_calls.get(), 0);
}
