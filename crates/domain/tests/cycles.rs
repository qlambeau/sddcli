//! Contract tests for relationship cycle detection.

use std::collections::BTreeSet;

use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, DocumentSnapshot, Metadata, MetadataValue,
    validate_relationship_cycles,
};
use proptest::prelude::*;
use rstest::rstest;

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

#[derive(Clone, Copy, Debug)]
enum SnapshotOrder {
    FirstSecondThird,
    SecondThirdFirst,
    ThirdFirstSecond,
}

fn ordered_snapshots(
    order: SnapshotOrder,
    first: &ArtifactSnapshot,
    second: &ArtifactSnapshot,
    third: &ArtifactSnapshot,
) -> Vec<ArtifactSnapshot> {
    match order {
        SnapshotOrder::FirstSecondThird => vec![first.clone(), second.clone(), third.clone()],
        SnapshotOrder::SecondThirdFirst => vec![second.clone(), third.clone(), first.clone()],
        SnapshotOrder::ThirdFirstSecond => vec![third.clone(), first.clone(), second.clone()],
    }
}

fn cycle_diagnostics(snapshots: &[ArtifactSnapshot]) -> Vec<domain::Diagnostic> {
    validate_relationship_cycles(snapshots)
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.CYCLE")
        .collect()
}

/// Covers: REQ-005 FR-001, FR-002, FR-006, FR-012, and FR-013 — empty and clean graphs succeed.
#[test]
fn returns_no_findings_for_empty_and_clean_graphs() {
    let clean = snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-001", &[]);

    assert!(cycle_diagnostics(&[]).is_empty());
    assert!(cycle_diagnostics(&[clean]).is_empty());
}

#[rstest]
#[case("depends_on")]
#[case("requires")]
#[case("blockers")]
/// Covers: REQ-005 FR-004 and FR-010 — every dependency relationship field participates in cycles.
fn detects_each_dependency_field_cycle(#[case] field: &str) {
    let first = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[(field, sequence(&["US-002"]))],
    );
    let second = snapshot(
        "specs/current/second-user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[(field, sequence(&["US-003"]))],
    );
    let third = snapshot(
        "specs/current/third-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[(field, sequence(&["US-001"]))],
    );

    let diagnostics = cycle_diagnostics(&[first, second, third]);

    assert_eq!(diagnostics.len(), 3, "all source entries should identify the cycle");
    assert!(diagnostics.iter().all(|diagnostic| diagnostic.message().contains(field)));
}

/// Covers: REQ-005 FR-005 and FR-010 — supersession directions normalize before traversal.
#[test]
fn normalizes_supersession_directions_into_one_cycle_graph() {
    let first = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("supersedes", MetadataValue::scalar("US-002"))],
    );
    let second =
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-002", &[]);
    let third = snapshot(
        "specs/archive/002-old/user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[
            ("superseded_by", MetadataValue::scalar("US-002")),
            ("supersedes", MetadataValue::scalar("US-001")),
        ],
    );

    let diagnostics = cycle_diagnostics(&[first, second, third]);

    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message().contains("supersedes")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message().contains("superseded_by")));
}

/// Covers: REQ-005 FR-007 and FR-010 — a target-valid self-loop receives one finding.
#[test]
fn reports_target_valid_self_loop() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["US-001"]))],
    );

    let diagnostics = cycle_diagnostics(&[source]);

    assert_eq!(diagnostics.len(), 1);
    let Some(diagnostic) = diagnostics.first() else { return };
    assert!(diagnostic.message().contains("US-001"));
}

/// Covers: REQ-005 FR-006 and FR-007 — wrong-kind self-loops remain outside the cycle graph.
#[test]
fn excludes_wrong_kind_self_loop() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("parent", MetadataValue::scalar("US-001"))],
    );

    assert!(cycle_diagnostics(&[source]).is_empty());
}

/// Covers: REQ-005 FR-002 — paths that cross relationship families do not form cycles.
#[test]
fn isolates_relationship_families() {
    let story = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("parent", MetadataValue::scalar("PRD-001"))],
    );
    let prd = snapshot(
        "specs/prds/PRD-001.md",
        ArtifactKind::Prd,
        "PRD-001",
        &[("depends_on", sequence(&["US-001"]))],
    );

    assert!(cycle_diagnostics(&[story, prd]).is_empty());
}

/// Covers: REQ-005 FR-006 and FR-011 — ineligible values do not receive cycle findings.
#[test]
fn excludes_ineligible_relationship_values() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[
            ("depends_on", sequence(&["", "TBD", "REQ-404"])),
            ("parent", MetadataValue::scalar("ADR-001")),
        ],
    );
    let wrong_kind_target = snapshot("specs/adr/ADR-001.md", ArtifactKind::Adr, "ADR-001", &[]);

    assert!(cycle_diagnostics(&[source, wrong_kind_target]).is_empty());
}

/// Covers: REQ-005 FR-008 and FR-009 — rotation-equivalent cycles collapse while overlapping cycles remain.
#[test]
fn deduplicates_rotations_and_retains_overlapping_cycles() {
    let first = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["US-002"]))],
    );
    let second = snapshot(
        "specs/current/second-user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("depends_on", sequence(&["US-003", "US-004"]))],
    );
    let third = snapshot(
        "specs/current/third-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("depends_on", sequence(&["US-001"]))],
    );
    let fourth = snapshot(
        "specs/current/fourth-user-story.md",
        ArtifactKind::UserStory,
        "US-004",
        &[("depends_on", sequence(&["US-001"]))],
    );

    let diagnostics =
        cycle_diagnostics(&[fourth.clone(), third.clone(), first.clone(), second.clone()]);
    let identities = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.message().split_once("cycle `"))
        .filter_map(|(_, identity)| identity.strip_suffix('`'))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    assert_eq!(diagnostics.len(), 6);
    assert_eq!(identities.len(), 2, "the two directed cycles need distinct identities");
    assert_eq!(diagnostics, cycle_diagnostics(&[first, second, third, fourth]));
}

/// Covers: REQ-005 FR-010 and FR-014 — findings remain source-owned and ordered deterministically.
#[test]
fn orders_source_owned_findings_deterministically() {
    let first = snapshot(
        "specs/current/z-user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("depends_on", sequence(&["US-001"]))],
    );
    let second = snapshot(
        "specs/current/a-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["US-002"]))],
    );

    let diagnostics = cycle_diagnostics(&[first, second]);

    assert_eq!(diagnostics.len(), 2);
    let Some(first_diagnostic) = diagnostics.first() else { return };
    let Some(second_diagnostic) = diagnostics.get(1) else { return };
    assert_eq!(first_diagnostic.path().as_str(), "specs/current/a-user-story.md");
    assert_eq!(second_diagnostic.path().as_str(), "specs/current/z-user-story.md");
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic.severity() == domain::Severity::Error
            && !diagnostic.remediation().is_empty()
            && diagnostic.message().contains("dependency")
    }));
}

proptest! {
    /// Covers: REQ-005 FR-008 and FR-014 — equivalent snapshot rotations preserve cycle identity and ordering.
    #[test]
    fn preserves_cycle_report_for_snapshot_rotations(
        order in prop::sample::select(vec![
            SnapshotOrder::FirstSecondThird,
            SnapshotOrder::SecondThirdFirst,
            SnapshotOrder::ThirdFirstSecond,
        ])
    ) {
        let first = snapshot(
            "specs/current/first-user-story.md",
            ArtifactKind::UserStory,
            "US-001",
            &[("depends_on", sequence(&["US-002"]))],
        );
        let second = snapshot(
            "specs/current/second-user-story.md",
            ArtifactKind::UserStory,
            "US-002",
            &[("depends_on", sequence(&["US-003"]))],
        );
        let third = snapshot(
            "specs/current/third-user-story.md",
            ArtifactKind::UserStory,
            "US-003",
            &[("depends_on", sequence(&["US-001"]))],
        );
        let expected = cycle_diagnostics(&[first.clone(), second.clone(), third.clone()]);
        let actual = ordered_snapshots(order, &first, &second, &third);

        prop_assert_eq!(cycle_diagnostics(&actual), expected);
    }
}
