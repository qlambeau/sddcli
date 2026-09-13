//! Integration coverage for pure Spec-Ready evaluation and schema contracts.

use proptest::prelude::*;

use domain::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, DocumentSnapshot, ImplementationPacketRef,
    Metadata, ScenarioCoverage, SpecReadyDecision, evaluate_spec_ready,
};

fn path(value: &str) -> ArtifactPath {
    match ArtifactPath::try_new(value) {
        Ok(path) => path,
        Err(_) => std::process::abort(),
    }
}

fn packet() -> ImplementationPacketRef {
    match ImplementationPacketRef::try_new("specs/008-evaluate-spec-ready-status") {
        Ok(packet) => packet,
        Err(_) => std::process::abort(),
    }
}

#[test]
fn recognizes_canonical_schema_artifact_contracts() {
    assert_eq!(ArtifactKind::from_path("specs/schema/DB-001.md"), Some(ArtifactKind::Database));
    assert_eq!(ArtifactKind::from_path("specs/schema/TABLE-001.md"), Some(ArtifactKind::Table));
    assert_eq!(ArtifactKind::Database.expected_type(), Some("database-schema"));
    assert_eq!(ArtifactKind::Table.expected_type(), Some("table-schema"));
    assert_eq!(ArtifactKind::Database.id_prefix(), Some("DB"));
    assert_eq!(ArtifactKind::Table.id_prefix(), Some("TABLE"));
}

#[test]
fn exposes_explicit_scenario_coverage_categories() {
    let feature = domain::FeatureSnapshot::new_with_coverage(
        Some("US-008".to_owned()),
        Some("approved".to_owned()),
        Some("Readiness".to_owned()),
        4,
        true,
        true,
        true,
        [
            ScenarioCoverage::Happy,
            ScenarioCoverage::Alternate,
            ScenarioCoverage::Failure,
            ScenarioCoverage::Boundary,
        ],
    );

    assert!(feature.coverage().contains(&ScenarioCoverage::Happy));
    assert!(feature.coverage().contains(&ScenarioCoverage::Alternate));
    assert!(feature.coverage().contains(&ScenarioCoverage::Failure));
    assert!(feature.coverage().contains(&ScenarioCoverage::Boundary));
}

#[test]
fn reports_missing_packet_artifacts_without_io_or_mutation() {
    let packet = packet();
    let before = Vec::<ArtifactSnapshot>::new();

    let result = evaluate_spec_ready(&packet, &before);

    assert!(matches!(result, SpecReadyDecision::NotReady(_)));
    let SpecReadyDecision::NotReady(result) = result else { return };
    assert!(!result.is_ready());
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "READINESS.PACKET.ARTIFACT_MISSING"
    }));
    assert_eq!(before, Vec::<ArtifactSnapshot>::new());
}

#[test]
fn missing_scenario_categories_are_reported_individually() {
    let packet = packet();
    let feature = domain::FeatureSnapshot::new_with_coverage(
        Some("US-008".to_owned()),
        Some("approved".to_owned()),
        Some("Readiness".to_owned()),
        1,
        true,
        true,
        true,
        [ScenarioCoverage::Happy],
    );
    let snapshot = ArtifactSnapshot::new(
        path("specs/008-evaluate-spec-ready-status/scenarios.feature"),
        ArtifactKind::Gherkin,
        Metadata::new(),
        DocumentSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Some(feature)),
    );

    let result = evaluate_spec_ready(&packet, &[snapshot]);

    let diagnostics = result.result().diagnostics();
    for category in ["Alternate:", "Failure:", "Boundary:"] {
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_id().as_str() == "READINESS.SCENARIO.COVERAGE"
                && diagnostic.message().contains(category)
        }));
    }
}

#[test]
fn accepts_a_complete_approved_packet_without_mutating_snapshots() {
    let packet = packet();
    let snapshots = complete_packet_snapshots();
    let before = snapshots.clone();

    let result = evaluate_spec_ready(&packet, &snapshots);

    assert!(
        result.is_ready(),
        "unexpected readiness diagnostics: {:?}",
        result.result().diagnostics()
    );
    assert!(result.result().diagnostics().is_empty());
    assert_eq!(result.result().packet(), &packet);
    assert_eq!(result.result().scope().len(), 5);
    let enriched = result.clone().with_diagnostics(Vec::new());
    assert!(enriched.is_ready());
    let enriched_result = result.result().clone().with_diagnostics(Vec::new());
    assert!(enriched_result.is_ready());
    assert_eq!(snapshots, before);
}

fn complete_packet_snapshots() -> Vec<ArtifactSnapshot> {
    vec![
        markdown("specs/prds/PRD-001.md", ArtifactKind::Prd, "PRD-001", "approved"),
        markdown(
            "specs/prds/PRD-001-epics/EPIC-003.md",
            ArtifactKind::Epic,
            "EPIC-003",
            "approved",
        ),
        markdown(
            "specs/008-evaluate-spec-ready-status/user-story.md",
            ArtifactKind::UserStory,
            "US-008",
            "approved",
        ),
        feature(),
        markdown(
            "specs/008-evaluate-spec-ready-status/requirements.md",
            ArtifactKind::Requirements,
            "REQ-008",
            "approved",
        ),
        markdown(
            "specs/008-evaluate-spec-ready-status/design.md",
            ArtifactKind::Design,
            "DES-008",
            "approved",
        ),
        markdown(
            "specs/008-evaluate-spec-ready-status/tasks.md",
            ArtifactKind::Task,
            "TASK-008",
            "approved",
        ),
    ]
}

#[allow(
    clippy::too_many_lines,
    reason = "the fixture names every artifact contract field and section"
)]
fn markdown(path_value: &str, kind: ArtifactKind, id: &str, status: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("title", "Complete fixture");
    let Some(type_value) = kind.expected_type() else { std::process::abort() };
    metadata.insert_scalar("type", type_value);
    metadata.insert_scalar("status", status);
    metadata.insert_scalar("created", "2026-09-13");
    metadata.insert_scalar("updated", "2026-09-13");
    metadata.insert_scalar("owner", "reviewer");
    match kind {
        ArtifactKind::Prd => {
            metadata.insert_scalar("scope", "project");
            metadata.insert("parent", domain::MetadataValue::Null);
            metadata.insert("supersedes", domain::MetadataValue::Null);
        }
        ArtifactKind::Epic => {
            metadata.insert_scalar("parent", "PRD-001");
            metadata.insert_sequence("depends_on", Vec::<String>::new());
            metadata.insert_sequence("requires", Vec::<String>::new());
            metadata.insert_sequence("blockers", Vec::<String>::new());
        }
        ArtifactKind::UserStory => {
            metadata.insert_scalar("parent", "PRD-001");
            metadata.insert_scalar("epic", "EPIC-003");
            metadata.insert_scalar("feature", "evaluate-readiness");
            metadata.insert_sequence("depends_on", Vec::<String>::new());
            metadata.insert_sequence("requires", Vec::<String>::new());
            metadata.insert_sequence("blockers", Vec::<String>::new());
        }
        ArtifactKind::Requirements | ArtifactKind::Design | ArtifactKind::Task => {
            metadata.insert_scalar("parent", "US-008");
            metadata.insert_sequence("depends_on", Vec::<String>::new());
            metadata.insert_sequence("requires", Vec::<String>::new());
            metadata.insert_sequence("blockers", Vec::<String>::new());
        }
        _ => {}
    }
    metadata.insert_sequence("related", Vec::<String>::new());
    let headings = match kind {
        ArtifactKind::Prd => vec![
            "Product Requirements Document",
            "Vision And Problem",
            "Target Personas And Journeys",
            "Success Metrics",
            "Functional Scope And Epics",
            "Non-Functional Requirements",
            "Assumptions And Out Of Scope",
            "Open Questions",
            "Decision Log",
            "Review Checklist",
        ],
        ArtifactKind::Epic => vec![
            "Epic Brief",
            "Outcome Statement",
            "Capability Boundaries",
            "Candidate Vertical Slices",
            "Success Criteria",
            "Dependencies",
            "Open Questions",
            "Readiness Checklist",
        ],
        ArtifactKind::UserStory => vec![
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
        ArtifactKind::Requirements => vec![
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
        ArtifactKind::Design => vec![
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
            "Errors",
        ],
        ArtifactKind::Task => vec![
            "Tasks",
            "Implementation Approach",
            "Ordered Tasks",
            "Test And Verification Plan",
            "Rollout And Recovery",
            "Definition Of Done",
        ],
        ArtifactKind::Gherkin
        | ArtifactKind::Adr
        | ArtifactKind::Database
        | ArtifactKind::Table => Vec::new(),
    };
    let lines = headings
        .iter()
        .enumerate()
        .map(|(index, heading)| domain::SourceLine::new(index + 1, format!("## {heading}")))
        .chain(
            (kind == ArtifactKind::Task)
                .then(|| {
                    vec![
                        domain::SourceLine::new(99, "RED test task"),
                        domain::SourceLine::new(100, "GREEN test task"),
                        domain::SourceLine::new(
                            101,
                            "Observed Verification Evidence: cargo xtask ci passed",
                        ),
                    ]
                })
                .into_iter()
                .flatten(),
        )
        .collect::<Vec<_>>();
    let normalized_headings = headings
        .iter()
        .enumerate()
        .map(|(index, heading)| domain::Heading::new(2, *heading, index + 1))
        .collect::<Vec<_>>();
    let checklist = if matches!(
        kind,
        ArtifactKind::Prd | ArtifactKind::Epic | ArtifactKind::UserStory | ArtifactKind::Task
    ) {
        vec![domain::ChecklistItem::new(true, "complete", 100)]
    } else {
        Vec::new()
    };
    ArtifactSnapshot::new(
        path(path_value),
        kind,
        metadata,
        DocumentSnapshot::new(lines, normalized_headings, checklist, None),
    )
}

fn replace_root_story(snapshots: &mut [ArtifactSnapshot], dependencies: &[&str]) {
    replace_root_story_with_links(snapshots, dependencies, &[]);
}

fn replace_root_story_with_links(
    snapshots: &mut [ArtifactSnapshot],
    dependencies: &[&str],
    requirements: &[&str],
) {
    let Some(index) = snapshots.iter().position(|snapshot| {
        snapshot.path().as_str() == "specs/008-evaluate-spec-ready-status/user-story.md"
    }) else {
        return;
    };
    let Some(base) = snapshots.get(index).cloned() else { return };
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", "US-008");
    metadata.insert_scalar("title", "Complete fixture");
    metadata.insert_scalar("type", "user-story");
    metadata.insert_scalar("status", "approved");
    metadata.insert_scalar("created", "2026-09-13");
    metadata.insert_scalar("updated", "2026-09-13");
    metadata.insert_scalar("owner", "reviewer");
    metadata.insert_scalar("parent", "PRD-001");
    metadata.insert_scalar("epic", "EPIC-003");
    metadata.insert_scalar("feature", "evaluate-readiness");
    metadata.insert_sequence("depends_on", dependencies.iter().copied());
    metadata.insert_sequence("requires", requirements.iter().copied());
    metadata.insert_sequence("blockers", Vec::<String>::new());
    metadata.insert_sequence("related", Vec::<String>::new());
    let Some(slot) = snapshots.get_mut(index) else { return };
    *slot = ArtifactSnapshot::new(
        base.path().clone(),
        ArtifactKind::UserStory,
        metadata,
        base.document().clone(),
    );
}

fn set_status(snapshots: &mut [ArtifactSnapshot], id: &str, status: &str) {
    let Some(index) = snapshots
        .iter()
        .position(|snapshot| snapshot.id().is_some_and(|value| value.as_str() == id))
    else {
        return;
    };
    let Some(base) = snapshots.get(index).cloned() else { return };
    let mut metadata = base.metadata().clone();
    metadata.insert_scalar("status", status);
    let Some(slot) = snapshots.get_mut(index) else { return };
    *slot =
        ArtifactSnapshot::new(base.path().clone(), base.kind(), metadata, base.document().clone());
}

fn add_root_related(snapshots: &mut [ArtifactSnapshot], related: &str) {
    let Some(index) = snapshots.iter().position(|snapshot| {
        snapshot.path().as_str() == "specs/008-evaluate-spec-ready-status/user-story.md"
    }) else {
        return;
    };
    let Some(base) = snapshots.get(index).cloned() else { return };
    let mut metadata = base.metadata().clone();
    metadata.insert_sequence("related", [related]);
    let Some(slot) = snapshots.get_mut(index) else { return };
    *slot =
        ArtifactSnapshot::new(base.path().clone(), base.kind(), metadata, base.document().clone());
}

fn add_root_blocker(snapshots: &mut [ArtifactSnapshot]) {
    let Some(index) = snapshots.iter().position(|snapshot| {
        snapshot.path().as_str() == "specs/008-evaluate-spec-ready-status/user-story.md"
    }) else {
        return;
    };
    let Some(base) = snapshots.get(index).cloned() else { return };
    let mut metadata = base.metadata().clone();
    metadata.insert_sequence("blockers", ["open blocker"]);
    let Some(slot) = snapshots.get_mut(index) else { return };
    *slot =
        ArtifactSnapshot::new(base.path().clone(), base.kind(), metadata, base.document().clone());
}

fn dependency_packet(
    root: &str,
    id: &str,
    status: &str,
    dependencies: &[&str],
) -> Vec<ArtifactSnapshot> {
    let story_path = format!("{root}/user-story.md");
    let mut story_metadata = Metadata::new();
    story_metadata.insert_scalar("id", id);
    story_metadata.insert_scalar("status", status);
    story_metadata.insert_sequence("depends_on", dependencies.iter().copied());
    let story = ArtifactSnapshot::new(
        path(&story_path),
        ArtifactKind::UserStory,
        story_metadata,
        DocumentSnapshot::empty(),
    );
    let feature = domain::FeatureSnapshot::new(
        Some(id.to_owned()),
        Some(status.to_owned()),
        Some("Dependency".to_owned()),
        1,
        true,
        true,
        true,
    );
    let gherkin = ArtifactSnapshot::new(
        path(&format!("{root}/scenarios.feature")),
        ArtifactKind::Gherkin,
        Metadata::new(),
        DocumentSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Some(feature)),
    );
    let requirements = dependency_artifact(root, ArtifactKind::Requirements, "REQ-009", status);
    let design = dependency_artifact(root, ArtifactKind::Design, "DES-009", status);
    let mut task = dependency_artifact(root, ArtifactKind::Task, "TASK-009", status);
    task = ArtifactSnapshot::new(
        task.path().clone(),
        task.kind(),
        task.metadata().clone(),
        DocumentSnapshot::new(
            vec![domain::SourceLine::new(1, "Observed Verification Evidence: passed")],
            Vec::new(),
            Vec::new(),
            None,
        ),
    );
    vec![story, gherkin, requirements, design, task]
}

fn dependency_artifact(root: &str, kind: ArtifactKind, id: &str, status: &str) -> ArtifactSnapshot {
    let file = match kind {
        ArtifactKind::Requirements => "requirements.md",
        ArtifactKind::Design => "design.md",
        ArtifactKind::Task => "tasks.md",
        _ => "user-story.md",
    };
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("status", status);
    ArtifactSnapshot::new(
        path(&format!("{root}/{file}")),
        kind,
        metadata,
        DocumentSnapshot::empty(),
    )
}

fn supporting_adr(id: &str) -> ArtifactSnapshot {
    let mut metadata = Metadata::new();
    metadata.insert_scalar("id", id);
    metadata.insert_scalar("title", "Supporting decision");
    metadata.insert_scalar("type", "architecture-decision-record");
    metadata.insert_scalar("status", "approved");
    metadata.insert_scalar("created", "2026-09-13");
    metadata.insert_scalar("updated", "2026-09-13");
    metadata.insert_scalar("owner", "reviewer");
    metadata.insert("supersedes", domain::MetadataValue::Null);
    metadata.insert("superseded_by", domain::MetadataValue::Null);
    metadata.insert_sequence("related", Vec::<String>::new());
    ArtifactSnapshot::new(
        path(&format!("specs/adr/{id}.md")),
        ArtifactKind::Adr,
        metadata,
        DocumentSnapshot::empty(),
    )
}

fn feature() -> ArtifactSnapshot {
    let feature = domain::FeatureSnapshot::new_with_coverage(
        Some("US-008".to_owned()),
        Some("approved".to_owned()),
        Some("Readiness".to_owned()),
        4,
        true,
        true,
        true,
        ScenarioCoverage::ALL,
    );
    ArtifactSnapshot::new(
        path("specs/008-evaluate-spec-ready-status/scenarios.feature"),
        ArtifactKind::Gherkin,
        Metadata::new(),
        DocumentSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Some(feature)),
    )
}

#[test]
fn includes_packet_supporting_artifacts_but_excludes_unrelated_artifacts() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    replace_root_story_with_links(&mut snapshots, &[], &["ADR-009"]);
    snapshots.push(supporting_adr("ADR-009"));
    snapshots.push(supporting_adr("ADR-010"));

    let result = evaluate_spec_ready(&packet, &snapshots);
    let scope = result.result().scope();

    assert!(scope.iter().any(|path| path.as_str() == "specs/adr/ADR-009.md"));
    assert!(!scope.iter().any(|path| path.as_str().contains("ADR-010")));

    add_root_related(&mut snapshots, "ADR-009");
    let duplicate_support = evaluate_spec_ready(&packet, &snapshots);
    assert!(
        duplicate_support
            .result()
            .scope()
            .iter()
            .any(|path| { path.as_str() == "specs/adr/ADR-009.md" })
    );
    replace_root_story_with_links(&mut snapshots, &[], &["ADR-404"]);
    let missing_support = evaluate_spec_ready(&packet, &snapshots);
    assert!(
        missing_support
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message().contains("ADR-404") })
    );

    replace_root_story_with_links(&mut snapshots, &[], &["ADR-009"]);
    set_status(&mut snapshots, "ADR-009", "draft");
    let unapproved_support = evaluate_spec_ready(&packet, &snapshots);
    assert!(
        unapproved_support
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message().contains("requires `approved`") })
    );
}

#[test]
fn an_artifact_beyond_approval_is_not_spec_ready() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    let Some(index) = snapshots.iter().position(|snapshot| {
        snapshot.path().as_str() == "specs/008-evaluate-spec-ready-status/tasks.md"
    }) else {
        return;
    };
    let Some(slot) = snapshots.get_mut(index) else { return };
    *slot = markdown(
        "specs/008-evaluate-spec-ready-status/tasks.md",
        ArtifactKind::Task,
        "TASK-008",
        "implemented",
    );

    let result = evaluate_spec_ready(&packet, &snapshots);

    assert!(!result.is_ready());
    assert!(result.result().diagnostics().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "READINESS.PACKET.LIFECYCLE"
            && diagnostic.path().as_str().ends_with("tasks.md")
    }));
}

#[test]
fn an_unimplemented_dependency_prevents_readiness() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    replace_root_story(&mut snapshots, &["US-009"]);
    snapshots.extend(dependency_packet("specs/009-blocked", "US-009", "approved", &[]));

    let result = evaluate_spec_ready(&packet, &snapshots);

    assert!(!result.is_ready());
    assert!(result.result().diagnostics().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "READINESS.DEPENDENCY.UNMET"
            && diagnostic.message().contains("not implemented")
    }));
}

#[test]
fn reports_one_deterministic_cycle_for_recursive_dependencies() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    replace_root_story(&mut snapshots, &["US-009"]);
    snapshots.extend(dependency_packet("specs/009-cycle", "US-009", "implemented", &["US-008"]));

    let first = evaluate_spec_ready(&packet, &snapshots);
    let second = evaluate_spec_ready(&packet, &snapshots);
    let first_cycles = first
        .result()
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.rule_id().as_str() == "READINESS.DEPENDENCY.CYCLE")
        .collect::<Vec<_>>();

    assert_eq!(first, second);
    assert_eq!(first_cycles.len(), 1);
}

#[test]
fn deduplicates_a_shared_dependency_reached_by_two_paths() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    replace_root_story(&mut snapshots, &["US-009", "US-010"]);
    snapshots.extend(dependency_packet("specs/009-first", "US-009", "implemented", &["US-011"]));
    snapshots.extend(dependency_packet("specs/010-second", "US-010", "implemented", &["US-011"]));
    snapshots.extend(dependency_packet("specs/011-shared", "US-011", "approved", &[]));

    let result = evaluate_spec_ready(&packet, &snapshots);
    let shared_findings = result
        .result()
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.path().as_str().contains("011-shared"))
        .count();

    assert!(!result.is_ready());
    assert_eq!(shared_findings, 5);
}

#[test]
fn readiness_is_invariant_to_snapshot_order() {
    let packet = packet();
    let snapshots = complete_packet_snapshots();
    let mut reversed = snapshots.clone();
    reversed.reverse();

    assert_eq!(evaluate_spec_ready(&packet, &snapshots), evaluate_spec_ready(&packet, &reversed));
}

proptest! {
    #[test]
    fn readiness_is_order_invariant_under_reordering(reverse in any::<bool>()) {
        let packet = packet();
        let mut snapshots = complete_packet_snapshots();
        if reverse {
            snapshots.reverse();
        }
        prop_assert_eq!(
            evaluate_spec_ready(&packet, &snapshots),
            evaluate_spec_ready(&packet, &complete_packet_snapshots())
        );
    }
}

#[test]
fn reports_missing_prerequisites_and_missing_dependency_targets() {
    let packet = packet();
    let mut story_metadata = Metadata::new();
    story_metadata.insert_scalar("id", "US-008");
    story_metadata.insert_scalar("status", "approved");
    story_metadata.insert_sequence("depends_on", ["US-999"]);
    let story = ArtifactSnapshot::new(
        path("specs/008-evaluate-spec-ready-status/user-story.md"),
        ArtifactKind::UserStory,
        story_metadata.clone(),
        DocumentSnapshot::empty(),
    );

    let result = evaluate_spec_ready(&packet, &[story]);

    assert!(
        result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str() == "READINESS.PACKET.PREDICATE" })
    );
    assert!(result.result().diagnostics().iter().any(|diagnostic| {
        diagnostic.rule_id().as_str() == "READINESS.DEPENDENCY.UNMET"
            && diagnostic.message().contains("US-999")
    }));

    let mut resolved_reference_metadata = story_metadata.clone();
    resolved_reference_metadata.insert_scalar("parent", "PRD-404");
    resolved_reference_metadata.insert_scalar("epic", "EPIC-404");
    resolved_reference_metadata
        .insert("blockers", domain::MetadataValue::Scalar("scalar blocker".to_owned()));
    let resolved_reference_story = ArtifactSnapshot::new(
        path("specs/008-evaluate-spec-ready-status/user-story.md"),
        ArtifactKind::UserStory,
        resolved_reference_metadata,
        DocumentSnapshot::empty(),
    );
    let resolved_result =
        evaluate_spec_ready(&packet, std::slice::from_ref(&resolved_reference_story));
    assert!(
        resolved_result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message().contains("PRD-404") })
    );
    assert!(
        resolved_result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message().contains("scalar blocker") })
    );

    let mut opaque_metadata = resolved_reference_story.metadata().clone();
    opaque_metadata.insert("blockers", domain::MetadataValue::Mapping);
    let opaque_story = ArtifactSnapshot::new(
        path("specs/008-evaluate-spec-ready-status/user-story.md"),
        ArtifactKind::UserStory,
        opaque_metadata,
        DocumentSnapshot::empty(),
    );
    let _ = evaluate_spec_ready(&packet, &[opaque_story]);
}

#[test]
fn reports_missing_dependency_artifacts_and_verification_evidence() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    replace_root_story(&mut snapshots, &["US-009"]);
    snapshots.extend(dependency_packet("specs/009-incomplete", "US-009", "implemented", &[]));
    let Some(task_index) = snapshots
        .iter()
        .position(|snapshot| snapshot.path().as_str() == "specs/009-incomplete/tasks.md")
    else {
        return;
    };
    let Some(task) = snapshots.get(task_index).cloned() else { return };
    let Some(slot) = snapshots.get_mut(task_index) else { return };
    *slot = ArtifactSnapshot::new(
        task.path().clone(),
        task.kind(),
        task.metadata().clone(),
        DocumentSnapshot::empty(),
    );
    snapshots.retain(|snapshot| snapshot.path().as_str() != "specs/009-incomplete/design.md");

    let result = evaluate_spec_ready(&packet, &snapshots);

    assert!(
        result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str() == "READINESS.DEPENDENCY.EVIDENCE" })
    );
    assert!(
        result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message().contains("missing packet artifact") })
    );
}

#[test]
fn reports_missing_tasks_and_design_contracts_and_blockers() {
    let packet = packet();
    let mut snapshots = complete_packet_snapshots();
    let Some(task_index) = snapshots.iter().position(|snapshot| {
        snapshot.path().as_str() == "specs/008-evaluate-spec-ready-status/tasks.md"
    }) else {
        return;
    };
    let Some(task) = snapshots.get(task_index).cloned() else { return };
    let Some(task_slot) = snapshots.get_mut(task_index) else { return };
    *task_slot = ArtifactSnapshot::new(
        task.path().clone(),
        task.kind(),
        task.metadata().clone(),
        DocumentSnapshot::empty(),
    );
    let Some(design_index) = snapshots.iter().position(|snapshot| {
        snapshot.path().as_str() == "specs/008-evaluate-spec-ready-status/design.md"
    }) else {
        return;
    };
    let Some(design) = snapshots.get(design_index).cloned() else { return };
    let Some(design_slot) = snapshots.get_mut(design_index) else { return };
    *design_slot = ArtifactSnapshot::new(
        design.path().clone(),
        design.kind(),
        design.metadata().clone(),
        DocumentSnapshot::empty(),
    );
    replace_root_story(&mut snapshots, &[]);
    add_root_blocker(&mut snapshots);

    let result = evaluate_spec_ready(&packet, &snapshots);
    assert!(
        result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.rule_id().as_str() == "READINESS.PACKET.BLOCKER" })
    );

    assert!(
        result
            .result()
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.message().contains("actionable RED") })
    );
    assert!(result.result().diagnostics().iter().any(|diagnostic| {
        diagnostic.message().contains("Interfaces And Contracts")
            || diagnostic.message().contains("interfaces and contracts")
    }));
}

#[test]
fn schema_link_diagnostics_are_bidirectional_and_deterministic() {
    let mut database_metadata = Metadata::new();
    database_metadata.insert_scalar("id", "DB-001");
    database_metadata.insert_scalar("status", "approved");
    database_metadata.insert_sequence("tables", ["TABLE-001", "TABLE-002"]);
    let database = ArtifactSnapshot::new(
        path("specs/schema/DB-001.md"),
        ArtifactKind::Database,
        database_metadata,
        DocumentSnapshot::empty(),
    );

    let mut table_metadata = Metadata::new();
    table_metadata.insert_scalar("id", "TABLE-001");
    table_metadata.insert_scalar("status", "approved");
    table_metadata.insert_scalar("database", "DB-001");
    let table = ArtifactSnapshot::new(
        path("specs/schema/TABLE-001.md"),
        ArtifactKind::Table,
        table_metadata,
        DocumentSnapshot::empty(),
    );

    let diagnostics = domain::validate_schema_links(&[database, table]);

    assert_eq!(diagnostics.len(), 1);
    let Some(diagnostic) = diagnostics.first() else { return };
    assert!(diagnostic.message().contains("TABLE-002"));
}

#[test]
fn validates_clean_and_non_reciprocal_schema_links_and_ignores_incomplete_metadata() {
    let mut database_metadata = Metadata::new();
    database_metadata.insert_scalar("id", "DB-001");
    database_metadata.insert_sequence("tables", ["TABLE-001"]);
    let database = ArtifactSnapshot::new(
        path("specs/schema/DB-001.md"),
        ArtifactKind::Database,
        database_metadata,
        DocumentSnapshot::empty(),
    );
    let mut table_metadata = Metadata::new();
    table_metadata.insert_scalar("id", "TABLE-001");
    table_metadata.insert_scalar("database", "DB-001");
    let table = ArtifactSnapshot::new(
        path("specs/schema/TABLE-001.md"),
        ArtifactKind::Table,
        table_metadata,
        DocumentSnapshot::empty(),
    );
    assert!(domain::validate_schema_links(&[database.clone(), table.clone()]).is_empty());

    let incomplete_database = ArtifactSnapshot::new(
        path("specs/schema/DB-002.md"),
        ArtifactKind::Database,
        Metadata::new(),
        DocumentSnapshot::empty(),
    );
    let incomplete_table = ArtifactSnapshot::new(
        path("specs/schema/TABLE-002.md"),
        ArtifactKind::Table,
        Metadata::new(),
        DocumentSnapshot::empty(),
    );
    assert!(domain::validate_schema_links(&[incomplete_database, incomplete_table]).is_empty());

    let mut missing_database_metadata = Metadata::new();
    missing_database_metadata.insert_scalar("id", "TABLE-003");
    missing_database_metadata.insert_scalar("database", "DB-404");
    let missing_database = ArtifactSnapshot::new(
        path("specs/schema/TABLE-003.md"),
        ArtifactKind::Table,
        missing_database_metadata,
        DocumentSnapshot::empty(),
    );
    let missing_diagnostics = domain::validate_schema_links(&[missing_database]);
    assert_eq!(missing_diagnostics.len(), 1);
    assert!(
        missing_diagnostics
            .first()
            .is_some_and(|diagnostic| diagnostic.message().contains("DB-404"))
    );

    let mut one_way_database_metadata = Metadata::new();
    one_way_database_metadata.insert_scalar("id", "DB-010");
    one_way_database_metadata.insert_sequence("tables", ["TABLE-010"]);
    let one_way_database = ArtifactSnapshot::new(
        path("specs/schema/DB-010.md"),
        ArtifactKind::Database,
        one_way_database_metadata,
        DocumentSnapshot::empty(),
    );
    let mut one_way_table_metadata = Metadata::new();
    one_way_table_metadata.insert_scalar("id", "TABLE-010");
    one_way_table_metadata.insert_scalar("database", "DB-011");
    let one_way_table = ArtifactSnapshot::new(
        path("specs/schema/TABLE-010.md"),
        ArtifactKind::Table,
        one_way_table_metadata,
        DocumentSnapshot::empty(),
    );
    let one_way = domain::validate_schema_links(&[one_way_database, one_way_table]);
    assert_eq!(one_way.len(), 2);
}
