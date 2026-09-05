//! Contract tests for repository-wide artifact identity validation.

use domain::{
    ArtifactId, ArtifactKind, ArtifactPath, ArtifactResult, ArtifactSnapshot, Diagnostic,
    DocumentSnapshot, Metadata, Severity, validate_identities,
};
use proptest::prelude::*;
use rstest::rstest;

fn snapshot(path: &str, kind: ArtifactKind, id: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    let Ok(path) = ArtifactPath::try_new(path) else { std::process::abort() };
    ArtifactSnapshot::new(path, kind, metadata, DocumentSnapshot::empty())
}

/// Covers: REQ-002 FR-001, FR-002, and FR-005 — unique and empty identity sets are clean.
#[test]
fn accepts_unique_and_empty_identity_sets() {
    let first = snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-001");
    let second = snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-002");

    let unique = validate_identities(&[first, second]);
    let empty = validate_identities(&[]);

    assert!(unique.is_empty());
    assert!(empty.is_empty());
}

/// Covers: REQ-002 FR-002 and FR-003 — every conflicting path receives a duplicate diagnostic.
#[test]
fn reports_duplicate_identity_for_every_conflicting_path() {
    let active = snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-001");
    let archived =
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-001");
    let unrelated =
        snapshot("specs/current/requirements.md", ArtifactKind::Requirements, "REQ-001");

    let diagnostics = validate_identities(&[active, archived, unrelated]);
    let duplicate_paths: Vec<&str> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.IDENTITY.DUPLICATE_ID")
        .map(|diagnostic| diagnostic.path().as_str())
        .collect();

    assert_eq!(
        duplicate_paths,
        vec!["specs/archive/001-old/user-story.md", "specs/current/user-story.md",]
    );
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic.message().contains("US-001") && diagnostic.remediation().contains("identifier")
    }));
}

/// Covers: REQ-002 FR-004 — path-encoded identities are compared with frontmatter identities.
#[rstest]
#[case(ArtifactKind::Prd, "specs/prds/PRD-003.md", "PRD-004", "PRD-003")]
#[case(ArtifactKind::Epic, "specs/prds/PRD-001-epics/EPIC-003.md", "EPIC-004", "EPIC-003")]
#[case(ArtifactKind::Adr, "specs/adr/ADR-003.md", "ADR-004", "ADR-003")]
fn reports_path_identity_mismatch(
    #[case] kind: ArtifactKind,
    #[case] path: &str,
    #[case] frontmatter_id: &str,
    #[case] filename_id: &str,
) {
    let artifact = snapshot(path, kind, frontmatter_id);

    let diagnostics = validate_identities(&[artifact]);

    assert!(
        diagnostics
            .iter()
            .find(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.IDENTITY.PATH_IDENTITY")
            .is_some()
    );
    let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.IDENTITY.PATH_IDENTITY")
    else {
        return;
    };
    assert!(diagnostic.message().contains(filename_id));
    assert!(diagnostic.message().contains(frontmatter_id));
    assert!(!diagnostic.remediation().is_empty());
}

/// Covers: REQ-002 FR-008 — identity findings are stable for identical inputs.
#[test]
fn produces_identical_diagnostics_for_identical_inputs() {
    let snapshots = [
        snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-001"),
        snapshot("specs/archive/001-old/user-story.md", ArtifactKind::UserStory, "US-001"),
        snapshot("specs/prds/PRD-003.md", ArtifactKind::Prd, "PRD-004"),
    ];

    let first = validate_identities(&snapshots);
    let second = validate_identities(&snapshots);

    assert_eq!(first, second);
}

// Covers: REQ-002 FR-002 and FR-007 — unrelated unique identities do not alter existing findings.
proptest! {
    #[test]
    fn retains_existing_findings_when_unrelated_identities_are_added(extra_count in 0usize..5) {
        let duplicate = [
            snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-001"),
            snapshot(
                "specs/archive/001-old/user-story.md",
                ArtifactKind::UserStory,
                "US-001",
            ),
        ];
        let expected = validate_identities(&duplicate);
        let mut expanded = duplicate.to_vec();
        for index in 0..extra_count {
            expanded.push(snapshot(
                &format!("specs/current/generated-{index}/user-story.md"),
                ArtifactKind::UserStory,
                &format!("US-{:03}", index + 10),
            ));
        }

        prop_assert_eq!(validate_identities(&expanded), expected);
    }
}

/// Covers: REQ-002 FR-003 and FR-007 — identity findings merge without replacing existing findings.
#[test]
fn appends_identity_diagnostics_to_an_artifact_result() {
    let artifact = snapshot("specs/current/user-story.md", ArtifactKind::UserStory, "US-001");
    let diagnostic = Diagnostic::new(
        artifact.path().clone(),
        None,
        "ARTIFACT.IDENTITY.DUPLICATE_ID",
        Severity::Error,
        "identifier US-001 is used by multiple artifacts",
        "assign a unique identifier",
    );
    let original = ArtifactResult::from_snapshot(&artifact);

    let merged = original.clone().with_diagnostics([diagnostic]);

    assert_eq!(merged.path(), original.path());
    assert_eq!(merged.id().map(ArtifactId::as_str), Some("US-001"));
    assert!(
        merged
            .violations()
            .iter()
            .any(|finding| { finding.rule_id().as_str() == "ARTIFACT.IDENTITY.DUPLICATE_ID" })
    );
    assert_eq!(merged.status(), domain::ArtifactStatus::Diagnostic);
}
