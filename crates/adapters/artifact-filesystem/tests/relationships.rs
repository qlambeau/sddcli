//! Integration tests for filesystem relationship target validation.

use std::fs;

use application::{RelationshipValidator, ValidationError};
use artifact_filesystem::FilesystemArtifactSource;

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    let Some(parent) = path.parent() else { std::process::abort() };
    assert!(fs::create_dir_all(parent).is_ok(), "test directory should be creatable");
    assert!(fs::write(path, contents).is_ok(), "test file should be writable");
}

fn frontmatter(id: &str) -> String {
    format!("---\nid: {id}\n---\n")
}

fn valid_user_story(id: &str, status: &str, requires: &[&str]) -> String {
    let requires =
        if requires.is_empty() { "[]".to_string() } else { format!("[{}]", requires.join(", ")) };
    format!(
        "---\n\
id: {id}\n\
title: Relationship fixture\n\
type: user-story\n\
status: {status}\n\
created: 2026-09-05\n\
updated: 2026-09-05\n\
owner: test-owner\n\
parent: PRD-001\n\
epic: EPIC-001\n\
feature: relationship\n\
depends_on: []\n\
requires: {requires}\n\
blockers: []\n\
related: []\n\
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

fn supporting_targets(root: &std::path::Path) {
    write_file(root, "specs/prds/PRD-001.md", &frontmatter("PRD-001"));
    write_file(root, "specs/prds/PRD-001-epics/EPIC-001.md", &frontmatter("EPIC-001"));
}

fn relationship_diagnostics(report: &domain::ValidationReport) -> Vec<&domain::Diagnostic> {
    report
        .artifacts()
        .iter()
        .flat_map(domain::ArtifactResult::violations)
        .filter(|diagnostic| diagnostic.rule_id().as_str().starts_with("ARTIFACT.RELATIONSHIP."))
        .collect()
}

fn result_for<'a>(
    report: &'a domain::ValidationReport,
    path: &str,
) -> Option<&'a domain::ArtifactResult> {
    report.artifacts().iter().find(|artifact| artifact.path().as_str() == path)
}

/// Covers: REQ-003 FR-001, FR-002, FR-003, and FR-005 — active and historical targets resolve by kind.
#[test]
fn resolves_active_archived_and_superseded_targets() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &valid_user_story("US-001", "approved", &["US-002", "US-003"]),
    );
    write_file(
        directory.path(),
        "specs/archive/001-old/user-story.md",
        &valid_user_story("US-002", "archived", &[]),
    );
    write_file(
        directory.path(),
        "specs/archive/002-old/user-story.md",
        &valid_user_story("US-003", "superseded", &[]),
    );

    let report =
        RelationshipValidator::new(FilesystemArtifactSource::new(directory.path())).validate();

    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert!(relationship_diagnostics(&report).is_empty());
    assert!(result_for(&report, "specs/archive/001-old/user-story.md").is_some());
    assert!(result_for(&report, "specs/archive/002-old/user-story.md").is_some());
}

/// Covers: REQ-003 FR-001 and FR-004 — excluded files cannot satisfy a target.
#[test]
fn excludes_template_and_supporting_file_targets() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &valid_user_story("US-001", "approved", &["REQ-900"]),
    );
    write_file(
        directory.path(),
        "specs/templates/feature/requirements.md",
        &frontmatter("REQ-900"),
    );
    write_file(directory.path(), "specs/supporting.md", &frontmatter("REQ-900"));

    let report =
        RelationshipValidator::new(FilesystemArtifactSource::new(directory.path())).validate();

    assert!(report.is_ok());
    let Ok(report) = report else { return };
    let Some(source) = result_for(&report, "specs/current/user-story.md") else { return };
    let diagnostics = relationship_diagnostics(&report);
    assert_eq!(diagnostics.len(), 1);
    let Some(diagnostic) = diagnostics.first() else { return };
    assert_eq!(diagnostic.path(), source.path());
    assert_eq!(diagnostic.rule_id().as_str(), "ARTIFACT.RELATIONSHIP.MISSING_TARGET");
    assert!(diagnostic.message().contains("REQ-900"));
}

/// Covers: REQ-003 FR-008 and FR-009 — empty, malformed, and unresolved values do not duplicate structural findings.
#[test]
fn leaves_structural_relationship_values_out_of_target_resolution() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    let malformed = valid_user_story("US-001", "approved", &["TBD", "not-an-id", ""]);
    write_file(directory.path(), "specs/current/user-story.md", &malformed);

    let report =
        RelationshipValidator::new(FilesystemArtifactSource::new(directory.path())).validate();

    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert!(relationship_diagnostics(&report).is_empty());
    let Some(source) = result_for(&report, "specs/current/user-story.md") else { return };
    assert!(source.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.USER-STORY.REFERENCE_SYNTAX"
    }));
}

/// Covers: REQ-003 FR-006 and FR-007 — every invalid entry is retained while unrelated artifacts remain represented.
#[test]
fn reports_multiple_invalid_entries_without_suppressing_unrelated_artifacts() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &valid_user_story("US-001", "approved", &["REQ-900", "REQ-901"]),
    );
    write_file(directory.path(), "specs/current/requirements.md", &frontmatter("REQ-001"));

    let report =
        RelationshipValidator::new(FilesystemArtifactSource::new(directory.path())).validate();

    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert_eq!(relationship_diagnostics(&report).len(), 2);
    assert!(result_for(&report, "specs/current/requirements.md").is_some());
}

/// Covers: REQ-003 FR-010 — repeated filesystem validation is identical and read-only.
#[test]
fn relationship_validation_is_deterministic_and_read_only() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    let source_path = directory.path().join("specs/current/user-story.md");
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &valid_user_story("US-001", "approved", &["REQ-900"]),
    );
    let Ok(before) = fs::read(&source_path) else { return };
    let source = FilesystemArtifactSource::new(directory.path());

    let first = RelationshipValidator::new(source.clone()).validate();
    let second = RelationshipValidator::new(source).validate();

    assert_eq!(first, second);
    let Ok(after) = fs::read(source_path) else { return };
    assert_eq!(after, before);
}

/// Covers: REQ-003 FR-010 — repository discovery failures remain typed and terminal.
#[test]
fn returns_typed_error_for_invalid_repository_root() {
    let Ok(directory) = tempfile::tempdir() else { return };
    let missing = directory.path().join("missing");

    let result = RelationshipValidator::new(FilesystemArtifactSource::new(missing)).validate();

    assert!(matches!(result, Err(ValidationError::Discovery(_))));
}
