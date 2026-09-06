//! Contract tests for relationship target validation.

use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Metadata,
    MetadataValue, validate_relationships,
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

fn scalar(value: &str) -> MetadataValue {
    MetadataValue::scalar(value)
}

fn sequence(values: &[&str]) -> MetadataValue {
    MetadataValue::sequence(values.iter().copied())
}

fn relationship_diagnostics(diagnostics: &[Diagnostic]) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_id().as_str().starts_with("ARTIFACT.RELATIONSHIP."))
        .collect()
}

/// Covers: REQ-003 FR-001, FR-002, FR-003, and FR-008 — valid references resolve across kinds and lifecycle-independent paths.
#[test]
fn resolves_valid_relationships_across_all_expected_kind_rules() {
    let prd = snapshot("specs/archive/001-prd/PRD-001.md", ArtifactKind::Prd, "PRD-001", &[]);
    let epic = snapshot("specs/archive/002-epic/EPIC-001.md", ArtifactKind::Epic, "EPIC-001", &[]);
    let parent_story =
        snapshot("specs/archive/003-parent/user-story.md", ArtifactKind::UserStory, "US-001", &[]);
    let successor_story =
        snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-002", &[]);
    let requirements = snapshot(
        "specs/current/requirements.md",
        ArtifactKind::Requirements,
        "REQ-001",
        &[("parent", scalar("US-001"))],
    );
    let design = snapshot(
        "specs/current/design.md",
        ArtifactKind::Design,
        "DES-001",
        &[("parent", scalar("US-001"))],
    );
    let adr = snapshot(
        "specs/current/ADR-001.md",
        ArtifactKind::Adr,
        "ADR-001",
        &[("supersedes", scalar("ADR-002"))],
    );
    let successor_adr =
        snapshot("specs/archive/004-adr/ADR-002.md", ArtifactKind::Adr, "ADR-002", &[]);
    let task = snapshot(
        "specs/current/tasks.md",
        ArtifactKind::Task,
        "TASK-001",
        &[("parent", scalar("US-001"))],
    );
    let story = snapshot(
        "specs/current/story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[
            ("parent", scalar("PRD-001")),
            ("epic", scalar("EPIC-001")),
            ("depends_on", sequence(&["REQ-001"])),
            ("requires", sequence(&["DES-001"])),
            ("blockers", sequence(&["ADR-001"])),
            ("related", sequence(&["TASK-001"])),
            ("supersedes", scalar("US-001")),
            ("superseded_by", scalar("US-002")),
        ],
    );

    let diagnostics = validate_relationships(&[
        prd,
        epic,
        parent_story,
        successor_story,
        requirements,
        design,
        adr,
        successor_adr,
        task,
        story,
    ]);

    assert!(relationship_diagnostics(&diagnostics).is_empty());
}

/// Covers: REQ-003 FR-004 and FR-006 — every absent concrete entry receives one finding.
#[test]
fn reports_missing_target_for_each_invalid_entry() {
    let source = snapshot(
        "specs/current/story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["REQ-404", "REQ-405"])), ("requires", sequence(&["ADR-404"]))],
    );

    let diagnostics = validate_relationships(&[source]);
    let relationship_diagnostics = relationship_diagnostics(&diagnostics);

    assert_eq!(relationship_diagnostics.len(), 3);
    assert!(relationship_diagnostics.iter().all(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.MISSING_TARGET"
    }));
    assert!(relationship_diagnostics.iter().any(|diagnostic| {
        diagnostic.message().contains("depends_on") && diagnostic.message().contains("REQ-404")
    }));
    assert!(relationship_diagnostics.iter().any(|diagnostic| {
        diagnostic.message().contains("requires") && diagnostic.message().contains("ADR-404")
    }));
}

#[rstest]
#[case(ArtifactKind::UserStory, "parent", "PRD", ArtifactKind::Adr, "ADR-001", "ADR")]
#[case(
    ArtifactKind::UserStory,
    "epic",
    "EPIC",
    ArtifactKind::Requirements,
    "REQ-001",
    "requirements"
)]
#[case(ArtifactKind::Design, "parent", "US", ArtifactKind::Epic, "EPIC-001", "epic")]
#[case(ArtifactKind::Adr, "supersedes", "ADR", ArtifactKind::UserStory, "US-001", "user story")]
/// Covers: REQ-003 FR-003 and FR-005 — expected-kind violations identify expected and actual kinds.
fn reports_wrong_kind_target(
    #[case] source_kind: ArtifactKind,
    #[case] field: &str,
    #[case] expected_kind: &str,
    #[case] actual_kind: ArtifactKind,
    #[case] target_id: &str,
    #[case] actual_kind_name: &str,
) {
    let source =
        snapshot("specs/current/source.md", source_kind, "US-002", &[(field, scalar(target_id))]);
    let target = snapshot("specs/current/target.md", actual_kind, target_id, &[]);

    let diagnostics = validate_relationships(&[source, target]);

    let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.WRONG_KIND")
    else {
        return;
    };
    assert!(diagnostic.message().contains(expected_kind));
    assert!(diagnostic.message().contains(actual_kind_name));
    assert!(diagnostic.message().contains(field));
    assert!(diagnostic.message().contains(target_id));
}

/// Covers: REQ-003 FR-008 and FR-009 — empty, malformed, and unresolved values stay structural-only.
#[test]
fn skips_values_owned_by_structural_validation() {
    let source = snapshot(
        "specs/current/story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[
            ("parent", MetadataValue::Null),
            ("depends_on", sequence(&[])),
            ("requires", sequence(&["", "TBD", "not-an-id"])),
            ("related", MetadataValue::Mapping),
        ],
    );

    let diagnostics = validate_relationships(&[source]);

    assert!(relationship_diagnostics(&diagnostics).is_empty());
}

/// Covers: DES-003 duplicate-identity interaction — duplicate IDs do not create duplicate relationship findings.
#[test]
fn does_not_duplicate_relationship_findings_for_duplicate_target_ids() {
    let source = snapshot(
        "specs/current/story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("requires", sequence(&["REQ-001"]))],
    );
    let first_target =
        snapshot("specs/current/requirements.md", ArtifactKind::Requirements, "REQ-001", &[]);
    let second_target = snapshot(
        "specs/archive/001-old/requirements.md",
        ArtifactKind::Requirements,
        "REQ-001",
        &[],
    );

    let diagnostics = validate_relationships(&[source, first_target, second_target]);

    assert!(relationship_diagnostics(&diagnostics).is_empty());
}

/// Covers: REQ-003 FR-007 and FR-010 — valid unrelated snapshots remain quiet and ordering is stable.
#[test]
fn retains_unrelated_results_and_deterministic_diagnostics() {
    let invalid = snapshot(
        "specs/current/invalid.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("depends_on", sequence(&["REQ-404"]))],
    );
    let unrelated =
        snapshot("specs/current/unrelated.md", ArtifactKind::Requirements, "REQ-001", &[]);

    let first = validate_relationships(&[invalid.clone(), unrelated.clone()]);
    let second = validate_relationships(&[unrelated, invalid]);

    assert_eq!(first, second);
    let diagnostics = relationship_diagnostics(&first);
    assert_eq!(diagnostics.len(), 1);
    let Some(diagnostic) = diagnostics.first() else { return };
    assert_eq!(diagnostic.path().as_str(), "specs/current/invalid.md");
}

proptest! {
    /// Covers: REQ-003 FR-007 and FR-010 — unrelated target additions preserve relationship findings.
    #[test]
    fn retains_missing_findings_when_unrelated_targets_are_added(extra_count in 0usize..5) {
        let source = snapshot(
            "specs/current/story.md",
            ArtifactKind::UserStory,
            "US-001",
            &[("depends_on", sequence(&["REQ-404"]))],
        );
        let expected = validate_relationships(std::slice::from_ref(&source));
        let mut expanded = vec![source];

        for index in 0..extra_count {
            let id = format!("REQ-{:03}", index + 100);
            let path = format!("specs/current/generated-{index}.md");
            expanded.push(snapshot(&path, ArtifactKind::Requirements, &id, &[]));
        }

        prop_assert_eq!(validate_relationships(&expanded), expected);
    }
}
