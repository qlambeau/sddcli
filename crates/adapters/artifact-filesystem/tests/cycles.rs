//! Scenario-equivalent filesystem tests for relationship cycle validation.

use std::fs;

use application::CycleValidator;
use artifact_filesystem::FilesystemArtifactSource;

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    let Some(parent) = path.parent() else { std::process::abort() };
    assert!(fs::create_dir_all(parent).is_ok(), "test directory should be creatable");
    assert!(fs::write(path, contents).is_ok(), "test file should be writable");
}

fn supporting_targets(root: &std::path::Path) {
    write_file(root, "specs/prds/PRD-001.md", "---\nid: PRD-001\n---\n");
    write_file(root, "specs/prds/PRD-001-epics/EPIC-001.md", "---\nid: EPIC-001\n---\n");
}

fn user_story(id: &str, dependency: &str) -> String {
    format!(
        "---\n\
id: {id}\n\
title: Cycle fixture\n\
type: user-story\n\
status: approved\n\
created: 2026-09-05\n\
updated: 2026-09-05\n\
owner: test-owner\n\
parent: PRD-001\n\
epic: EPIC-001\n\
feature: cycle\n\
depends_on: [{dependency}]\n\
requires: []\n\
blockers: []\n\
related: []\n\
supersedes: null\n\
superseded_by: null\n\
---\n\
## User Story\n\
## Story Card\n\
## Context And Value\n\
## Business Rules\n\
## Examples\n\
## Acceptance Criteria\n\
## Scope Boundaries\n\
## Dependencies\n\
## Open Questions\n\
## Invest Check\n\
- [x] complete\n"
    )
}

fn cycle_diagnostics(report: &domain::ValidationReport) -> Vec<&domain::Diagnostic> {
    report
        .artifacts()
        .iter()
        .flat_map(domain::ArtifactResult::violations)
        .filter(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.CYCLE")
        .collect()
}

/// Covers: REQ-005 FR-001, FR-004, FR-010, and FR-012 — real discovery validates active and historical cycles.
#[test]
fn detects_cycles_across_active_and_archived_artifacts() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(directory.path(), "specs/current/user-story.md", &user_story("US-001", "US-002"));
    write_file(
        directory.path(),
        "specs/archive/001-old/user-story.md",
        &user_story("US-002", "US-003"),
    );
    write_file(
        directory.path(),
        "specs/archive/002-superseded/user-story.md",
        &user_story("US-003", "US-001"),
    );

    let result = CycleValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    assert!(result.is_ok());
    let Ok(report) = result else { return };

    assert_eq!(cycle_diagnostics(&report).len(), 3);
    assert_eq!(report.artifacts().len(), 5);
}

/// Covers: REQ-005 FR-006, FR-011, and FR-014 — excluded files cannot create edges and validation is read-only.
#[test]
fn preserves_missing_target_and_repository_bytes_for_excluded_values() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    let source_relative = "specs/current/user-story.md";
    write_file(directory.path(), source_relative, &user_story("US-001", "REQ-900"));
    write_file(
        directory.path(),
        "specs/templates/feature/requirements.md",
        "---\nid: REQ-900\n---\n",
    );
    write_file(directory.path(), "specs/supporting.md", "---\nid: REQ-900\n---\n");
    let source_path = directory.path().join(source_relative);
    let Ok(before) = fs::read(&source_path) else { return };

    let first = CycleValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    let second = CycleValidator::new(FilesystemArtifactSource::new(directory.path())).validate();

    assert_eq!(first, second);
    let Ok(first) = first else { return };
    assert!(cycle_diagnostics(&first).is_empty());
    assert!(first.artifacts().iter().flat_map(domain::ArtifactResult::violations).any(
        |diagnostic| { diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.MISSING_TARGET" }
    ));
    let Ok(after) = fs::read(source_path) else { return };
    assert_eq!(after, before);
}
