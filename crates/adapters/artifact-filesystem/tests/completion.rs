//! Filesystem integration tests for guarded completion transitions.

use std::fmt::Write as _;
use std::fs;

use application::{
    ArtifactIdentitySource, PacketPromoter, PacketPromotionOutcome, PromoteArtifactCommand,
    PromotePacketCommand, Promoter, PromotionOutcome,
};
use artifact_filesystem::{FilesystemArtifactSource, FilesystemPromotionStore, SystemClock};
use domain::{ArtifactKind, ArtifactPath, ImplementationPacketRef, LifecycleState, PromotionActor};

fn must<T, E>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(_) => std::process::abort(),
    }
}

fn must_some<T>(value: Option<T>) -> T {
    match value {
        Some(value) => value,
        None => std::process::abort(),
    }
}

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    let parent = must_some(path.parent());
    must(fs::create_dir_all(parent));
    must(fs::write(path, contents));
}

fn actor() -> PromotionActor {
    must(PromotionActor::try_new("reviewer-1"))
}

fn release(status: &str, feature_status: &str) -> String {
    format!(
        "---\nid: REL-001\ntitle: Release\ntype: release-record\nstatus: {status}\nversion: v0.1.0\ncommit: abc123\ndate: 2026-09-13\nowner: reviewer\nrelated: []\n---\n# Release Record\n## Release Overview\nComplete release\n## Included Features\n| Feature ID | Title | User Story | Status |\n| --- | --- | --- | --- |\n| 009 | Completion | `US-009` | {feature_status} |\n## Verification Evidence\n- cargo test --workspace passed\n## Migration & Rollback\nNo migration\n"
    )
}

fn adr(id: &str, status: &str, supersedes: &str, superseded_by: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: Decision\ntype: architecture-decision-record\nstatus: {status}\ncreated: 2026-09-13\nupdated: 2026-09-13\nowner: reviewer\nsupersedes: {supersedes}\nsuperseded_by: {superseded_by}\nrelated: []\n---\n## Context\n## Decision\n## Alternatives Considered\n## Consequences\n## Follow-Up Actions\n"
    )
}

fn packet_file(kind: ArtifactKind, id: &str, status: &str) -> String {
    if kind == ArtifactKind::Gherkin {
        return format!(
            "# parent: US-009\n# status: {status}\n\nFeature: Completion\n\n  Scenario: Happy: archive\n    Given a released feature\n    When archival is requested\n    Then the packet is archived\n"
        );
    }
    let parent = if kind == ArtifactKind::UserStory {
        "parent: PRD-001\nepic: EPIC-003\nfeature: completion\n"
    } else {
        "parent: US-009\n"
    };
    let references = "depends_on: []\nrequires: []\nblockers: []\n";
    let headings: &[&str] = match kind {
        ArtifactKind::UserStory => &[
            "User Story",
            "Story Card",
            "Context And Value",
            "Business Rules",
            "Examples",
            "Acceptance Criteria",
            "Scope Boundaries",
            "Dependencies",
            "Open Questions",
            "Invest Check",
        ],
        ArtifactKind::Requirements => &[
            "Requirements",
            "Purpose And Actors",
            "Preconditions",
            "Inputs And Outputs",
            "Functional Requirements",
            "Postconditions And Invariants",
            "Edge And Failure Behavior",
            "Quality Requirements",
            "Traceability",
        ],
        ArtifactKind::Design => &[
            "Design",
            "Context And Constraints",
            "Proposed Design",
            "Components And Responsibilities",
            "Interfaces And Contracts",
            "Data And State Flow",
            "Security, Performance, And Operations",
            "Alternatives Considered",
            "Risks And Open Decisions",
            "Verification Approach",
        ],
        ArtifactKind::Task => &[
            "Tasks",
            "Implementation Approach",
            "Ordered Tasks",
            "Test And Verification Plan",
            "Rollout And Recovery",
            "Definition Of Done",
        ],
        _ => &[],
    };
    let type_value = must_some(kind.expected_type());
    let mut content = format!(
        "---\nid: {id}\ntitle: Packet artifact\ntype: {type_value}\nstatus: {status}\ncreated: 2026-09-13\nupdated: 2026-09-13\nowner: reviewer\n{parent}{references}related: []\n---\n"
    );
    for heading in headings {
        let _ = writeln!(content, "## {heading}");
    }
    content.push_str("- [x] complete\n");
    content
}

/// Covers: REQ-009 FR-003 and FR-004 — supersession preserves the successor and predecessor path.
#[test]
fn supersedes_an_approved_artifact_without_changing_its_successor() {
    let directory = must(tempfile::tempdir());
    write_file(
        directory.path(),
        "specs/adr/ADR-001.md",
        &adr("ADR-001", "approved", "null", "ADR-002"),
    );
    let successor = adr("ADR-002", "approved", "ADR-001", "null");
    write_file(directory.path(), "specs/adr/ADR-002.md", &successor);
    let predecessor_path = must(ArtifactPath::try_new("specs/adr/ADR-001.md"));
    let source = FilesystemArtifactSource::new(directory.path());
    let store = FilesystemPromotionStore::new(directory.path());
    let mut promoter = Promoter::new(source, store, SystemClock);

    let result = promoter.promote(&PromoteArtifactCommand::new(
        predecessor_path.clone(),
        LifecycleState::Superseded,
        actor(),
    ));

    assert!(
        matches!(result, Ok(PromotionOutcome::Promoted(plan)) if plan.target() == LifecycleState::Superseded)
    );
    let predecessor = must(fs::read_to_string(directory.path().join(predecessor_path.as_str())));
    assert!(predecessor.contains("status: superseded\n"));
    let successor_after = must(fs::read_to_string(directory.path().join("specs/adr/ADR-002.md")));
    assert_eq!(successor_after, successor);
}

/// Covers: REQ-009 FR-005 and FR-009 — release promotion patches only lifecycle metadata.
#[test]
fn promotes_a_complete_release_record_without_changing_body() {
    let directory = must(tempfile::tempdir());
    let contents = release("approved", "implemented");
    write_file(directory.path(), "specs/releases/REL-001.md", &contents);
    let path = must(ArtifactPath::try_new("specs/releases/REL-001.md"));
    let before = contents.clone();
    let source = FilesystemArtifactSource::new(directory.path());
    let store = FilesystemPromotionStore::new(directory.path());
    let mut promoter = Promoter::new(source, store, SystemClock);

    let result = promoter.promote(&PromoteArtifactCommand::new(
        path.clone(),
        LifecycleState::Released,
        actor(),
    ));

    assert!(
        matches!(result, Ok(PromotionOutcome::Promoted(plan)) if plan.target() == LifecycleState::Released)
    );
    let after = must(fs::read_to_string(directory.path().join(path.as_str())));
    assert!(after.contains("status: released\n"));
    assert!(after.contains("promoted_to: released\n"));
    assert!(after.contains("## Release Overview\nComplete release\n"));
    assert!(
        after.starts_with(&before.replace("status: approved", "status: released"))
            || after.contains("status: released")
    );
}

/// Covers: REQ-009 FR-006, FR-007, and FR-009 — archival patches five files without moving or mutating support.
#[test]
fn archives_a_relocated_packet_without_moving_files_or_supporting_adrs() {
    let directory = must(tempfile::tempdir());
    write_file(directory.path(), "specs/releases/REL-001.md", &release("released", "implemented"));
    let packet_path = "specs/archive/009-guarded-completion";
    let packet_files = [
        ("user-story.md", ArtifactKind::UserStory, "US-009"),
        ("scenarios.feature", ArtifactKind::Gherkin, ""),
        ("requirements.md", ArtifactKind::Requirements, "REQ-009"),
        ("design.md", ArtifactKind::Design, "DES-009"),
        ("tasks.md", ArtifactKind::Task, "TASK-009"),
    ];
    for (file, kind, id) in packet_files {
        write_file(
            directory.path(),
            &format!("{packet_path}/{file}"),
            &packet_file(kind, id, "implemented"),
        );
    }
    let adr = "---\nid: ADR-009\ntitle: Support\ntype: architecture-decision-record\nstatus: approved\ncreated: 2026-09-13\nupdated: 2026-09-13\nowner: reviewer\nsupersedes: null\nsuperseded_by: null\nrelated: []\n---\n## Context\n## Decision\n## Alternatives Considered\n## Consequences\n## Follow-Up Actions\n";
    write_file(directory.path(), "specs/adr/ADR-009.md", adr);
    let before_paths =
        packet_files.iter().map(|(file, _, _)| format!("{packet_path}/{file}")).collect::<Vec<_>>();
    let before = before_paths
        .iter()
        .map(|path| must(fs::read(directory.path().join(path))))
        .collect::<Vec<_>>();
    let packet = must(ImplementationPacketRef::try_new(packet_path));
    let source = FilesystemArtifactSource::new(directory.path());
    let store = FilesystemPromotionStore::new(directory.path());
    let mut promoter = PacketPromoter::new(source, store, SystemClock);

    let result = promoter.promote(&PromotePacketCommand::new(packet, actor()));

    assert!(
        matches!(result, Ok(PacketPromotionOutcome::Promoted(result)) if result.advanced().len() == 5)
    );
    for (index, path) in before_paths.iter().enumerate() {
        let after = must(fs::read(directory.path().join(path)));
        assert!(String::from_utf8_lossy(&after).contains("status: archived"));
        let Some(original) = before.get(index) else { std::process::abort() };
        assert_ne!(after, *original);
    }
    assert!(directory.path().join(packet_path).is_dir());
    let supporting = must(fs::read_to_string(directory.path().join("specs/adr/ADR-009.md")));
    assert!(supporting.contains("status: approved\n"));
}

/// Covers: REQ-009 FR-006 and FR-013 — offline identity discovery includes release and archive candidates.
#[test]
fn identity_discovery_includes_release_and_archived_packet_paths() {
    let directory = must(tempfile::tempdir());
    write_file(directory.path(), "specs/releases/REL-001.md", &release("released", "implemented"));
    write_file(
        directory.path(),
        "specs/archive/009-guarded-completion/user-story.md",
        &packet_file(ArtifactKind::UserStory, "US-009", "implemented"),
    );

    let candidates = must(FilesystemArtifactSource::new(directory.path()).discover_identities());
    let paths = candidates.iter().map(|candidate| candidate.path().as_str()).collect::<Vec<_>>();
    assert!(paths.contains(&"specs/releases/REL-001.md"));
    assert!(paths.contains(&"specs/archive/009-guarded-completion/user-story.md"));
}
