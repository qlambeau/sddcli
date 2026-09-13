//! Completion-transition domain contract tests.

use proptest::prelude::*;

use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, DocumentSnapshot, ImplementationPacketRef,
    LifecycleState, Metadata, MetadataValue, PromotionActor, PromotionDecision, PromotionRequest,
    PromotionTimestamp, ReleaseFeature, ReleaseSnapshot, ScenarioCoverage, decide_promotion,
    plan_packet_archival,
};

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

fn artifact_snapshot(
    path_value: &str,
    kind: ArtifactKind,
    id: &str,
    status: &str,
    superseded_by: Option<&str>,
    supersedes: Option<&str>,
) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("status", status);
    metadata.insert_scalar("type", kind.expected_type().unwrap_or(""));
    if let Some(value) = superseded_by {
        metadata.insert_scalar("superseded_by", value);
    }
    if let Some(value) = supersedes {
        metadata.insert_scalar("supersedes", value);
    }
    ArtifactSnapshot::new(path(path_value), kind, metadata, DocumentSnapshot::empty())
}

fn release_snapshot(status: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", "REL-001");
    metadata.insert_scalar("status", status);
    metadata.insert_scalar("type", "release-record");
    ArtifactSnapshot::new(
        path("specs/releases/REL-001.md"),
        ArtifactKind::Release,
        metadata,
        DocumentSnapshot::empty().with_release(ReleaseSnapshot::new(
            vec![ReleaseFeature::new("US-009", "implemented")],
            true,
            true,
        )),
    )
}

/// Covers: REQ-009 FR-002 and FR-008 — release identity and terminal released state are normalized.
#[test]
fn recognizes_release_artifacts_and_terminal_released_state() {
    assert_eq!(ArtifactKind::from_path("specs/releases/REL-001.md"), Some(ArtifactKind::Release));
    assert_eq!(ArtifactKind::Release.token(), "release");
    assert_eq!(ArtifactKind::Release.expected_type(), Some("release-record"));
    assert_eq!(ArtifactKind::Release.id_prefix(), Some("REL"));
    assert_eq!(LifecycleState::parse("released"), Some(LifecycleState::Released));
    assert!(!LifecycleState::Released.is_active());
}

/// Covers: REQ-009 FR-003 and FR-004 — approved reciprocal successors permit supersession.
#[test]
fn accepts_supersession_only_for_an_approved_reciprocal_successor() {
    let predecessor = artifact_snapshot(
        "specs/adr/ADR-001.md",
        ArtifactKind::Adr,
        "ADR-001",
        "approved",
        Some("ADR-002"),
        None,
    );
    let successor = artifact_snapshot(
        "specs/adr/ADR-002.md",
        ArtifactKind::Adr,
        "ADR-002",
        "approved",
        None,
        Some("ADR-001"),
    );
    let request = PromotionRequest::new(LifecycleState::Superseded, actor());

    let decision = decide_promotion(
        &predecessor,
        &domain::completion_facts(
            &predecessor,
            &[predecessor.clone(), successor],
            LifecycleState::Superseded,
        ),
        &request,
        PromotionTimestamp::from_unix_seconds(42),
    );

    assert!(
        matches!(decision, PromotionDecision::Accepted(plan) if plan.target() == LifecycleState::Superseded)
    );
}

/// Covers: REQ-009 FR-009 through FR-011 — supersession guards reject malformed and unresolved successors.
#[test]
fn rejects_malformed_unapproved_and_nonreciprocal_successors() {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", "ADR-001");
    metadata.insert_scalar("status", "approved");
    metadata.insert_scalar("type", "adr");
    metadata.insert("superseded_by", MetadataValue::Sequence(vec!["ADR-002".to_string()]));
    let predecessor = ArtifactSnapshot::new(
        path("specs/adr/ADR-001.md"),
        ArtifactKind::Adr,
        metadata,
        DocumentSnapshot::empty(),
    );
    let malformed = decide_promotion(
        &predecessor,
        &domain::completion_facts(
            &predecessor,
            std::slice::from_ref(&predecessor),
            LifecycleState::Superseded,
        ),
        &PromotionRequest::new(LifecycleState::Superseded, actor()),
        PromotionTimestamp::from_unix_seconds(42),
    );
    assert!(
        matches!(malformed, PromotionDecision::Rejected(diagnostics) if diagnostics.iter().any(|item| item.rule_id().as_str().ends_with("SUPERSESSION_SUCCESSOR_MISSING")))
    );

    let predecessor = artifact_snapshot(
        "specs/adr/ADR-001.md",
        ArtifactKind::Adr,
        "ADR-001",
        "approved",
        Some("ADR-002"),
        None,
    );
    let successor = artifact_snapshot(
        "specs/adr/ADR-002.md",
        ArtifactKind::Adr,
        "ADR-002",
        "implemented",
        None,
        None,
    );
    let facts = domain::completion_facts(
        &predecessor,
        &[predecessor.clone(), successor],
        LifecycleState::Superseded,
    );
    let decision = decide_promotion(
        &predecessor,
        &facts,
        &PromotionRequest::new(LifecycleState::Superseded, actor()),
        PromotionTimestamp::from_unix_seconds(42),
    );
    assert!(
        matches!(decision, PromotionDecision::Rejected(diagnostics) if diagnostics.iter().any(|item| item.rule_id().as_str().ends_with("SUPERSESSION_SUCCESSOR_STATE")) && diagnostics.iter().any(|item| item.rule_id().as_str().ends_with("SUPERSESSION_LINKS")))
    );
}

/// Covers: REQ-009 FR-009 through FR-011 — missing supersession prerequisites aggregate deterministically.
#[test]
fn rejects_supersession_with_all_missing_prerequisites() {
    let predecessor = artifact_snapshot(
        "specs/adr/ADR-001.md",
        ArtifactKind::Adr,
        "ADR-001",
        "approved",
        None,
        None,
    );
    let request = PromotionRequest::new(LifecycleState::Superseded, actor());
    let facts = domain::completion_facts(
        &predecessor,
        std::slice::from_ref(&predecessor),
        LifecycleState::Superseded,
    );

    let decision =
        decide_promotion(&predecessor, &facts, &request, PromotionTimestamp::from_unix_seconds(42));
    let PromotionDecision::Rejected(diagnostics) = decision else {
        assert!(matches!(decision, PromotionDecision::Rejected(_)));
        return;
    };
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rule_id().as_str().ends_with("SUPERSESSION_SUCCESSOR_MISSING")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rule_id().as_str().ends_with("SUPERSESSION_PREREQUISITE")
    }));
}

/// Covers: REQ-009 FR-008 — terminal released records reject further promotion.
#[test]
fn rejects_a_terminal_release_record() {
    let snapshot = release_snapshot("released");
    let request = PromotionRequest::new(LifecycleState::Approved, actor());
    let decision = decide_promotion(
        &snapshot,
        &domain::completion_facts(
            &snapshot,
            std::slice::from_ref(&snapshot),
            LifecycleState::Approved,
        ),
        &request,
        PromotionTimestamp::from_unix_seconds(42),
    );
    assert!(
        matches!(decision, PromotionDecision::Rejected(diagnostics) if diagnostics.iter().any(|diagnostic| diagnostic.rule_id().as_str().ends_with("TERMINAL_STATE")))
    );
}

/// Covers: REQ-009 FR-005 — complete release evidence permits release promotion.
#[test]
fn accepts_a_complete_release_record_as_released() {
    let request = PromotionRequest::new(LifecycleState::Released, actor());

    let decision = decide_promotion(
        &release_snapshot("approved"),
        &domain::completion_facts(
            &release_snapshot("approved"),
            &[release_snapshot("approved")],
            LifecycleState::Released,
        ),
        &request,
        PromotionTimestamp::from_unix_seconds(42),
    );

    assert!(
        matches!(decision, PromotionDecision::Accepted(plan) if plan.target() == LifecycleState::Released)
    );
}

/// Covers: REQ-009 FR-006 and FR-007 — incomplete packet archival remains a pure rejection.
#[test]
fn archives_only_a_relocated_packet_with_a_released_record() {
    let Ok(packet) = ImplementationPacketRef::try_new("specs/archive/009-guarded-completion")
    else {
        std::process::abort();
    };
    let story = packet_snapshot(
        "specs/archive/009-guarded-completion/user-story.md",
        ArtifactKind::UserStory,
        "US-009",
    );
    let release = release_snapshot("released");

    let decision = plan_packet_archival(
        &packet,
        &[story],
        &[release],
        &actor(),
        PromotionTimestamp::from_unix_seconds(42),
    );

    assert!(matches!(decision, domain::PacketPromotionDecision::Rejected(_)));
}

fn packet_snapshot(path_value: &str, kind: ArtifactKind, id: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("status", "implemented");
    metadata.insert_scalar("type", kind.expected_type().unwrap_or(""));
    ArtifactSnapshot::new(path(path_value), kind, metadata, DocumentSnapshot::empty())
}

/// Covers: REQ-009 FR-006 — archived packet references use the canonical archive path.
#[test]
fn accepts_archived_packet_reference() {
    let Ok(packet) = ImplementationPacketRef::try_new("specs/archive/009-guarded-completion")
    else {
        std::process::abort();
    };
    assert_eq!(packet.colocated_paths().len(), 5);
}

proptest! {
    /// Covers: REQ-009 FR-011 and FR-012 — completion facts are invariant to snapshot ordering.
    #[test]
    fn completion_facts_are_snapshot_order_invariant(reverse in any::<bool>()) {
        let predecessor = artifact_snapshot(
            "specs/adr/ADR-001.md",
            ArtifactKind::Adr,
            "ADR-001",
            "approved",
            Some("ADR-002"),
            None,
        );
        let successor = artifact_snapshot(
            "specs/adr/ADR-002.md",
            ArtifactKind::Adr,
            "ADR-002",
            "approved",
            None,
            Some("ADR-001"),
        );
        let mut snapshots = vec![predecessor.clone(), successor];
        let first = domain::completion_facts(
            &predecessor,
            &snapshots,
            LifecycleState::Superseded,
        );
        if reverse {
            snapshots.reverse();
        }
        let second = domain::completion_facts(
            &predecessor,
            &snapshots,
            LifecycleState::Superseded,
        );
        prop_assert_eq!(first, second);
    }
}

/// Covers: REQ-009 FR-001 — scenario coverage prefixes remain exact and case-sensitive.
#[test]
fn classifies_exact_scenario_coverage_prefixes() {
    assert_eq!(ScenarioCoverage::from_name("Happy: works"), Some(ScenarioCoverage::Happy));
    assert_eq!(ScenarioCoverage::from_name("Alternate: works"), Some(ScenarioCoverage::Alternate));
    assert_eq!(ScenarioCoverage::from_name("Failure: works"), Some(ScenarioCoverage::Failure));
    assert_eq!(ScenarioCoverage::from_name("Boundary: works"), Some(ScenarioCoverage::Boundary));
    assert_eq!(ScenarioCoverage::from_name("happy: works"), None);
    assert_eq!(
        ScenarioCoverage::ALL.map(ScenarioCoverage::prefix),
        ["Happy:", "Alternate:", "Failure:", "Boundary:"]
    );
}

/// Covers: REQ-009 FR-005 — normalized release rows preserve feature status and evidence.
#[test]
fn release_snapshot_preserves_feature_rows() {
    let release =
        ReleaseSnapshot::new(vec![ReleaseFeature::new("US-009", "implemented")], true, true);
    let Some(feature) = release.features().first() else { return };
    assert_eq!(feature.story_id(), "US-009");
    assert_eq!(feature.status(), "implemented");
    assert!(release.has_verification_evidence());
    assert!(release.has_release_commit());
}
