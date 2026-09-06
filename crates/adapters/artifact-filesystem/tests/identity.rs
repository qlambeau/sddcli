//! Integration tests for repository-wide filesystem identity discovery.

use std::fs;

use application::{ArtifactCandidate, ArtifactIdentitySource, IdentityValidator, ValidationError};
use artifact_filesystem::FilesystemArtifactSource;
use domain::{ArtifactStatus, OverallStatus};

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    let Some(parent) = path.parent() else { std::process::abort() };
    assert!(fs::create_dir_all(parent).is_ok(), "test directory should be creatable");
    assert!(fs::write(path, contents).is_ok(), "test file should be writable");
}

fn frontmatter(id: &str) -> String {
    format!("---\nid: {id}\n---\n")
}

fn valid_user_story(id: &str, status: &str) -> String {
    format!(
        "---\n\
id: {id}\n\
title: Identity fixture\n\
type: user-story\n\
status: {status}\n\
created: 2026-09-05\n\
updated: 2026-09-05\n\
owner: test-owner\n\
parent: PRD-001\n\
epic: EPIC-001\n\
feature: identity\n\
depends_on: []\n\
requires: []\n\
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

fn result_for<'a>(
    report: &'a domain::ValidationReport,
    path: &str,
) -> Option<&'a domain::ArtifactResult> {
    report.artifacts().iter().find(|artifact| artifact.path().as_str() == path)
}

#[derive(Clone)]
struct StaticIdentitySource {
    candidates: Vec<ArtifactCandidate>,
}

impl ArtifactIdentitySource for StaticIdentitySource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        Ok(self.candidates.clone())
    }
}

fn assert_identity_source_contract<S: ArtifactIdentitySource>(source: &S, expected_paths: &[&str]) {
    let first = source.discover_identities();
    assert!(first.is_ok());
    let Ok(first) = first else { return };
    let first_paths: Vec<&str> = first.iter().map(|candidate| candidate.path().as_str()).collect();
    assert_eq!(first_paths, expected_paths);

    let second = source.discover_identities();
    assert!(second.is_ok());
    let Ok(second) = second else { return };
    assert_eq!(first, second);
}

/// Covers: REQ-002 FR-001, FR-005, and FR-008 — identity scope includes archives but excludes templates/supporting files.
#[test]
fn discovers_active_and_archived_identity_paths_only() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/current/user-story.md", &frontmatter("US-001"));
    write_file(directory.path(), "specs/archive/001-old/user-story.md", &frontmatter("US-002"));
    write_file(directory.path(), "specs/templates/feature/user-story.md", &frontmatter("US-003"));
    write_file(directory.path(), "specs/current/supporting.md", &frontmatter("US-004"));

    let candidates = FilesystemArtifactSource::new(directory.path()).discover_identities();
    assert!(candidates.is_ok());
    let Ok(candidates) = candidates else { return };
    let paths: Vec<&str> = candidates.iter().map(|candidate| candidate.path().as_str()).collect();

    assert_eq!(paths, vec!["specs/archive/001-old/user-story.md", "specs/current/user-story.md",]);
}

/// Covers: DES-002 and R-TST-14 — each identity-source implementation has the same observable contract.
#[test]
fn identity_source_contract_holds_for_filesystem_and_in_memory_sources() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/current/user-story.md", &frontmatter("US-001"));
    write_file(directory.path(), "specs/archive/001-old/user-story.md", &frontmatter("US-002"));
    let expected_paths = ["specs/archive/001-old/user-story.md", "specs/current/user-story.md"];
    let filesystem = FilesystemArtifactSource::new(directory.path());
    let candidates = filesystem.discover_identities();
    assert!(candidates.is_ok());
    let Ok(candidates) = candidates else { return };

    assert_identity_source_contract(&filesystem, &expected_paths);
    let in_memory = StaticIdentitySource { candidates };
    assert_identity_source_contract(&in_memory, &expected_paths);
}

/// Covers: REQ-002 FR-001, FR-002, and FR-005 — unique active, archived, and superseded identities succeed.
#[test]
fn accepts_unique_current_and_historical_identities() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(
        directory.path(),
        "specs/current/user-story.md",
        &valid_user_story("US-001", "approved"),
    );
    write_file(
        directory.path(),
        "specs/archive/001-old/user-story.md",
        &valid_user_story("US-002", "superseded"),
    );

    let report = IdentityValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };

    assert_eq!(report.status(), OverallStatus::Success);
    assert_eq!(report.artifacts().len(), 2);
    assert!(report.artifacts().iter().all(|artifact| {
        artifact.status() == ArtifactStatus::Ok && artifact.violations().is_empty()
    }));
}

/// Covers: REQ-002 FR-002, FR-003, and FR-007 — duplicate IDs fail every conflicting result and preserve unrelated results.
#[test]
fn reports_duplicate_ids_across_active_and_archived_files() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/current/user-story.md", &frontmatter("US-001"));
    write_file(directory.path(), "specs/archive/001-old/user-story.md", &frontmatter("US-001"));
    write_file(directory.path(), "specs/current/requirements.md", &frontmatter("REQ-001"));

    let report = IdentityValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };

    assert_eq!(report.status(), OverallStatus::Failure);
    for path in ["specs/current/user-story.md", "specs/archive/001-old/user-story.md"] {
        assert!(result_for(&report, path).is_some());
        let Some(result) = result_for(&report, path) else { return };
        assert_eq!(result.status(), ArtifactStatus::Diagnostic);
        assert!(result.violations().iter().any(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.IDENTITY.DUPLICATE_ID"
                && diagnostic.message().contains("US-001")
        }));
    }
    assert!(
        report
            .artifacts()
            .iter()
            .any(|artifact| { artifact.path().as_str() == "specs/current/requirements.md" })
    );
}

/// Covers: REQ-002 FR-004 — every path-encoded PRD, epic, and ADR mismatch remains diagnosable.
#[test]
fn reports_path_identity_mismatches_from_real_files() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/prds/PRD-003.md", &frontmatter("PRD-004"));
    write_file(directory.path(), "specs/prds/PRD-001-epics/EPIC-003.md", &frontmatter("EPIC-004"));
    write_file(directory.path(), "specs/adr/ADR-003.md", &frontmatter("ADR-004"));

    let report = IdentityValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };

    assert_eq!(report.status(), OverallStatus::Failure);
    for (path, filename_id, frontmatter_id) in [
        ("specs/prds/PRD-003.md", "PRD-003", "PRD-004"),
        ("specs/prds/PRD-001-epics/EPIC-003.md", "EPIC-003", "EPIC-004"),
        ("specs/adr/ADR-003.md", "ADR-003", "ADR-004"),
    ] {
        assert!(result_for(&report, path).is_some());
        let Some(result) = result_for(&report, path) else { return };
        assert!(result.violations().iter().any(|diagnostic| {
            diagnostic.rule_id().as_str() == "ARTIFACT.IDENTITY.PATH_IDENTITY"
                && diagnostic.message().contains(filename_id)
                && diagnostic.message().contains(frontmatter_id)
        }));
    }
}

/// Covers: REQ-002 FR-006 — malformed frontmatter remains structural and contributes no identity.
#[test]
fn preserves_malformed_frontmatter_as_structural_diagnostic() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/current/user-story.md", "---\nid: [\n---\n# User Story\n");

    let report = IdentityValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert!(result_for(&report, "specs/current/user-story.md").is_some());
    let Some(result) = result_for(&report, "specs/current/user-story.md") else { return };

    assert!(
        result
            .violations()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str().ends_with("FRONTMATTER_PARSE") })
    );
    assert!(
        !result
            .violations()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str().starts_with("ARTIFACT.IDENTITY.") })
    );
}

/// Covers: REQ-002 FR-005 — no eligible identity paths produce an empty success.
#[test]
fn discovers_empty_identity_set_successfully() {
    let Ok(directory) = tempfile::tempdir() else { return };

    let report = IdentityValidator::new(FilesystemArtifactSource::new(directory.path())).validate();
    assert!(report.is_ok());
    let Ok(report) = report else { return };

    assert_eq!(report.status(), OverallStatus::Success);
    assert!(report.artifacts().is_empty());
}

/// Covers: REQ-002 FR-008 — repeated identity validation is deterministic and read-only.
#[test]
fn identity_validation_is_deterministic_and_read_only() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/current/user-story.md", &frontmatter("US-001"));
    write_file(directory.path(), "specs/archive/001-old/user-story.md", &frontmatter("US-001"));
    let current = directory.path().join("specs/current/user-story.md");
    let archived = directory.path().join("specs/archive/001-old/user-story.md");
    let Ok(before_current) = fs::read(&current) else { return };
    let Ok(before_archived) = fs::read(&archived) else { return };
    let source = FilesystemArtifactSource::new(directory.path());

    let first = IdentityValidator::new(source.clone()).validate();
    let second = IdentityValidator::new(source).validate();

    assert_eq!(first, second);
    let Ok(after_current) = fs::read(current) else { return };
    let Ok(after_archived) = fs::read(archived) else { return };
    assert_eq!(after_current, before_current);
    assert_eq!(after_archived, before_archived);
}
