//! Contract tests for reciprocal relationship validation.

use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot, Metadata,
    MetadataValue, validate_reciprocal_relationships,
};

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

fn scalar(value: &str) -> MetadataValue {
    MetadataValue::scalar(value)
}

fn non_reciprocal(diagnostics: &[Diagnostic]) -> Vec<&Diagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.NON_RECIPROCAL"
        })
        .collect()
}

/// Covers: REQ-004 FR-001, FR-002, and FR-003 — related membership resolves across active and historical artifacts.
#[test]
fn accepts_reciprocal_related_links_across_lifecycle_states() {
    let active = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("related", sequence(&["US-002"]))],
    );
    let archived = snapshot(
        "specs/archive/001-old/user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("related", sequence(&["US-001"]))],
    );

    let diagnostics = validate_reciprocal_relationships(&[active, archived]);

    assert!(non_reciprocal(&diagnostics).is_empty());
}

/// Covers: REQ-004 FR-004, FR-006, and FR-007 — every unmatched directed related entry is reported on its source.
#[test]
fn reports_each_missing_related_counterpart_and_continues() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("related", sequence(&["US-002"]))],
    );
    let target = snapshot(
        "specs/archive/001-old/user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("related", sequence(&["US-003"]))],
    );
    let reciprocal_target = snapshot(
        "specs/archive/002-old/user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("related", sequence(&["US-002"]))],
    );

    let diagnostics = validate_reciprocal_relationships(&[source, target, reciprocal_target]);
    let findings = non_reciprocal(&diagnostics);

    assert_eq!(findings.len(), 1);
    let Some(finding) = findings.first() else { return };
    assert_eq!(finding.path().as_str(), "specs/current/user-story.md");
    assert!(finding.message().contains("related"));
    assert!(finding.message().contains("US-002"));
    assert!(finding.message().contains("reverse"));
    assert!(finding.remediation().contains("US-002"));
}

/// Covers: REQ-004 FR-005 — both supersession directions require their mapped reverse field.
#[test]
fn reports_missing_counterparts_for_both_supersession_directions() {
    let superseder = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("supersedes", scalar("US-002"))],
    );
    let superseded_by = snapshot(
        "specs/archive/001-old/user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("superseded_by", scalar("US-004"))],
    );
    let target =
        snapshot("specs/archive/002-old/user-story.md", ArtifactKind::UserStory, "US-002", &[]);
    let successor =
        snapshot("specs/current/second-user-story.md", ArtifactKind::UserStory, "US-004", &[]);

    let diagnostics =
        validate_reciprocal_relationships(&[superseder, superseded_by, target, successor]);
    let findings = non_reciprocal(&diagnostics);

    assert_eq!(findings.len(), 2);
    assert!(findings.iter().any(|finding| {
        finding.message().contains("supersedes")
            && finding.message().contains("US-002")
            && finding.message().contains("superseded_by")
    }));
    assert!(findings.iter().any(|finding| {
        finding.message().contains("superseded_by")
            && finding.message().contains("US-004")
            && finding.message().contains("supersedes")
    }));
}

/// Covers: REQ-004 FR-002, FR-003, and FR-005 — reciprocal supersession works in both directions.
#[test]
fn accepts_both_reciprocal_supersession_directions() {
    let superseder = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("supersedes", scalar("US-002"))],
    );
    let historical = snapshot(
        "specs/archive/001-old/user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("superseded_by", scalar("US-001"))],
    );
    let successor = snapshot(
        "specs/current/second-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("superseded_by", scalar("US-004"))],
    );
    let predecessor = snapshot(
        "specs/archive/002-old/user-story.md",
        ArtifactKind::UserStory,
        "US-004",
        &[("supersedes", scalar("US-003"))],
    );

    let diagnostics =
        validate_reciprocal_relationships(&[superseder, historical, successor, predecessor]);

    assert!(non_reciprocal(&diagnostics).is_empty());
}

/// Covers: REQ-004 FR-002, FR-008, and FR-009 — malformed and unresolved values remain outside reciprocity.
#[test]
fn skips_structurally_invalid_and_unresolved_values() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[
            ("related", scalar("US-002")),
            ("supersedes", sequence(&["US-003"])),
            ("superseded_by", scalar("TBD")),
            ("requires", sequence(&["REQ-404"])),
        ],
    );

    let diagnostics = validate_reciprocal_relationships(&[source]);

    assert!(non_reciprocal(&diagnostics).is_empty());
}

/// Covers: REQ-004 FR-002 and FR-008 — unresolved, missing, and wrong-kind targets are skipped.
#[test]
fn skips_unresolved_missing_and_wrong_kind_targets() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[
            ("related", sequence(&["US-404", "TBD", "not-an-id"])),
            ("supersedes", scalar("REQ-001")),
        ],
    );
    let wrong_kind =
        snapshot("specs/current/requirements.md", ArtifactKind::Requirements, "REQ-001", &[]);

    let diagnostics = validate_reciprocal_relationships(&[source, wrong_kind]);

    assert!(non_reciprocal(&diagnostics).is_empty());
}

/// Covers: REQ-004 FR-003 and FR-006 — reverse membership ignores order and duplicate occurrences.
#[test]
fn accepts_reordered_and_duplicate_reverse_membership() {
    let source = snapshot(
        "specs/current/user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("related", sequence(&["US-002"]))],
    );
    let target = snapshot(
        "specs/archive/001-old/user-story.md",
        ArtifactKind::UserStory,
        "US-002",
        &[("related", sequence(&["US-003", "US-001", "US-001"]))],
    );
    let unrelated = snapshot(
        "specs/current/other-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("related", sequence(&["US-002"]))],
    );

    let diagnostics = validate_reciprocal_relationships(&[source, target, unrelated]);

    assert!(non_reciprocal(&diagnostics).is_empty());
}

/// Covers: REQ-004 FR-006 and FR-010 — input order does not change complete diagnostic ordering.
#[test]
fn produces_deterministic_diagnostics_independent_of_snapshot_order() {
    let first = snapshot(
        "specs/current/first-user-story.md",
        ArtifactKind::UserStory,
        "US-001",
        &[("related", sequence(&["US-002"]))],
    );
    let second = snapshot(
        "specs/current/second-user-story.md",
        ArtifactKind::UserStory,
        "US-003",
        &[("supersedes", scalar("US-004"))],
    );
    let targets = [
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-002", &[]),
        snapshot("specs/archive/002-old/user-story.md", ArtifactKind::UserStory, "US-004", &[]),
    ];

    let first_result = validate_reciprocal_relationships(&[
        first.clone(),
        second.clone(),
        targets[0].clone(),
        targets[1].clone(),
    ]);
    let second_result =
        validate_reciprocal_relationships(&[targets[1].clone(), targets[0].clone(), second, first]);

    assert_eq!(first_result, second_result);
}
