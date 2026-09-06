//! Scenario-equivalent filesystem tests for reciprocal relationship validation.

use std::fs;

use application::ReciprocalRelationshipValidator;
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

fn user_story(
    id: &str,
    status: &str,
    related: &[&str],
    supersedes: Option<&str>,
    superseded_by: Option<&str>,
) -> String {
    let related = format!("[{}]", related.join(", "));
    let supersedes = supersedes.map_or_else(|| "null".to_string(), str::to_string);
    let superseded_by = superseded_by.map_or_else(|| "null".to_string(), str::to_string);
    format!(
        "---\n\
id: {id}\n\
title: Reciprocal fixture\n\
type: user-story\n\
status: {status}\n\
created: 2026-09-05\n\
updated: 2026-09-05\n\
owner: test-owner\n\
parent: PRD-001\n\
epic: EPIC-001\n\
feature: reciprocal\n\
depends_on: []\n\
requires: []\n\
blockers: []\n\
related: {related}\n\
supersedes: {supersedes}\n\
superseded_by: {superseded_by}\n\
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

fn non_reciprocal(report: &domain::ValidationReport) -> Vec<&domain::Diagnostic> {
    report
        .artifacts()
        .iter()
        .flat_map(domain::ArtifactResult::violations)
        .filter(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.NON_RECIPROCAL"
        })
        .collect()
}

/// Covers: REQ-004 FR-001, FR-003, and FR-009 — reciprocal links resolve across active, archived, and superseded files.
#[test]
fn resolves_related_and_supersession_links_across_historical_scope() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &user_story("US-001", "approved", &["US-002"], Some("US-003"), None),
    );
    write_file(
        directory.path(),
        "specs/archive/001-old/user-story.md",
        &user_story("US-002", "archived", &["US-001"], None, None),
    );
    write_file(
        directory.path(),
        "specs/archive/002-superseded/user-story.md",
        &user_story("US-003", "superseded", &[], None, Some("US-001")),
    );

    let result =
        ReciprocalRelationshipValidator::new(FilesystemArtifactSource::new(directory.path()))
            .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert!(non_reciprocal(&report).is_empty());
    assert_eq!(report.artifacts().len(), 5);
}

/// Covers: REQ-004 FR-004, FR-005, FR-006, and FR-007 — each asymmetric directed entry gets one source diagnostic.
#[test]
fn reports_independent_missing_related_and_supersession_counterparts() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &user_story("US-001", "approved", &["US-002"], Some("US-002"), None),
    );
    write_file(
        directory.path(),
        "specs/archive/001-old/user-story.md",
        &user_story("US-002", "archived", &[], None, None),
    );

    let result =
        ReciprocalRelationshipValidator::new(FilesystemArtifactSource::new(directory.path()))
            .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    let findings = non_reciprocal(&report);
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().all(|finding| {
        finding.path().as_str() == "specs/current/user-story.md"
            && !finding.message().is_empty()
            && !finding.remediation().is_empty()
    }));
}

/// Covers: REQ-004 FR-001, FR-002, and FR-008 — templates and supporting files cannot satisfy unresolved targets.
#[test]
fn excludes_template_and_supporting_files_from_reciprocity() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &user_story("US-001", "approved", &["REQ-900"], None, None),
    );
    write_file(
        directory.path(),
        "specs/templates/feature/requirements.md",
        "---\nid: REQ-900\n---\n",
    );
    write_file(directory.path(), "specs/supporting.md", "---\nid: REQ-900\n---\n");

    let result =
        ReciprocalRelationshipValidator::new(FilesystemArtifactSource::new(directory.path()))
            .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert!(non_reciprocal(&report).is_empty());
    let source = report
        .artifacts()
        .iter()
        .find(|artifact| artifact.path().as_str() == "specs/current/user-story.md");
    assert!(source.is_some());
    let Some(source) = source else { return };
    assert!(source.violations().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "ARTIFACT.RELATIONSHIP.MISSING_TARGET"
    }));
}

/// Covers: REQ-004 FR-008 and FR-009 — malformed and empty relationship values receive no reciprocity findings.
#[test]
fn skips_empty_and_structurally_invalid_relationship_values() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    let contents = user_story("US-001", "approved", &[], None, None)
        .replace("related: []", "related: US-002")
        .replace("supersedes: null", "supersedes: [US-003]")
        .replace("superseded_by: null", "superseded_by: TBD");
    write_file(directory.path(), "specs/current/user-story.md", &contents);

    let result =
        ReciprocalRelationshipValidator::new(FilesystemArtifactSource::new(directory.path()))
            .validate();

    assert!(result.is_ok());
    let Ok(report) = result else { return };
    assert!(non_reciprocal(&report).is_empty());
}

/// Covers: REQ-004 FR-006 and FR-010 — duplicate reverse values and repeated reads are stable and read-only.
#[test]
fn ignores_reverse_order_and_preserves_repository_bytes() {
    let Ok(directory) = tempfile::tempdir() else { return };
    supporting_targets(directory.path());
    let source_relative = "specs/current/user-story.md";
    let target_relative = "specs/archive/001-old/user-story.md";
    write_file(
        directory.path(),
        source_relative,
        &user_story("US-001", "approved", &["US-002"], None, None),
    );
    write_file(
        directory.path(),
        target_relative,
        &user_story("US-002", "superseded", &["US-003", "US-001", "US-001"], None, None),
    );
    let source_path = directory.path().join(source_relative);
    let target_path = directory.path().join(target_relative);
    let Ok(before_source) = fs::read(&source_path) else { return };
    let Ok(before_target) = fs::read(&target_path) else { return };
    let source = FilesystemArtifactSource::new(directory.path());

    let first = ReciprocalRelationshipValidator::new(source.clone()).validate();
    let second = ReciprocalRelationshipValidator::new(source).validate();

    assert_eq!(first, second);
    let Ok(after_source) = fs::read(source_path) else { return };
    let Ok(after_target) = fs::read(target_path) else { return };
    assert_eq!(after_source, before_source);
    assert_eq!(after_target, before_target);
}
