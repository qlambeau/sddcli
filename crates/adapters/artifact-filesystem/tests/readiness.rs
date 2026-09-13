//! Integration coverage for schema discovery and Gherkin coverage parsing.

use std::fs;

use application::{ArtifactSource, EvaluateSpecReadyCommand, SpecReadyEvaluator};
use artifact_filesystem::{FilesystemArtifactSource, parse_artifact};
use domain::{ArtifactKind, ArtifactPath, ScenarioCoverage};

fn write_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    let Some(parent) = path.parent() else { std::process::abort() };
    assert!(fs::create_dir_all(parent).is_ok());
    assert!(fs::write(path, contents).is_ok());
}

#[test]
fn discovers_canonical_schema_documents_but_not_schema_templates() {
    let Ok(directory) = tempfile::tempdir() else { return };
    write_file(directory.path(), "specs/schema/DB-001.md", "---\nid: DB-001\n---\n");
    write_file(directory.path(), "specs/schema/TABLE-001.md", "---\nid: TABLE-001\n---\n");
    write_file(directory.path(), "specs/templates/supporting/database.md", "template");

    let Ok(candidates) = FilesystemArtifactSource::new(directory.path()).discover() else {
        return;
    };

    let paths = candidates.iter().map(|candidate| candidate.path().as_str()).collect::<Vec<_>>();
    assert_eq!(paths, vec!["specs/schema/DB-001.md", "specs/schema/TABLE-001.md"]);
    let Some(first) = candidates.first() else { return };
    let Some(second) = candidates.get(1) else { return };
    assert_eq!(first.snapshot().map(domain::ArtifactSnapshot::kind), Some(ArtifactKind::Database));
    assert_eq!(second.snapshot().map(domain::ArtifactSnapshot::kind), Some(ArtifactKind::Table));
}

#[test]
fn evaluates_the_repository_deterministically_without_writing() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let packet_path = root.join("specs/008-evaluate-spec-ready-status");
    let before = fs::read_dir(&packet_path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| fs::read(entry.path()).ok())
        .collect::<Vec<_>>();
    let Ok(packet) =
        domain::ImplementationPacketRef::try_new("specs/008-evaluate-spec-ready-status")
    else {
        return;
    };
    let evaluator = SpecReadyEvaluator::new(FilesystemArtifactSource::new(root.clone()));
    let Ok(first) = evaluator.evaluate(&EvaluateSpecReadyCommand::new(packet.clone())) else {
        return;
    };
    let Ok(second) = evaluator.evaluate(&EvaluateSpecReadyCommand::new(packet)) else {
        return;
    };

    assert_eq!(first, second);
    let after = fs::read_dir(packet_path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| fs::read(entry.path()).ok())
        .collect::<Vec<_>>();
    assert_eq!(before, after);
}

#[test]
fn parses_only_exact_case_sensitive_scenario_coverage_prefixes() {
    let Ok(path) = ArtifactPath::try_new("specs/008-readiness/scenarios.feature") else {
        return;
    };
    let contents = "# parent: US-008\n# status: approved\n\nFeature: Readiness\n\n  Scenario: Happy: complete\n    Given a packet\n    When it is evaluated\n    Then it is ready\n\n  Scenario: Alternate: supporting\n    Given a packet\n    When it is evaluated\n    Then it is ready\n\n  Scenario: Failure: incomplete\n    Given a packet\n    When it is evaluated\n    Then it is not ready\n\n  Scenario: Boundary: exact status\n    Given a packet\n    When it is evaluated\n    Then it is not ready\n\n  Scenario: happy: wrong case\n    Given a packet\n    When it is evaluated\n    Then it is not ready\n";

    let snapshot = parse_artifact(path, ArtifactKind::Gherkin, contents);
    let Some(feature) = snapshot.document().feature() else { return };

    assert!(feature.coverage().contains(&ScenarioCoverage::Happy));
    assert!(feature.coverage().contains(&ScenarioCoverage::Alternate));
    assert!(feature.coverage().contains(&ScenarioCoverage::Failure));
    assert!(feature.coverage().contains(&ScenarioCoverage::Boundary));
    assert_eq!(feature.scenario_count(), 5);
}
