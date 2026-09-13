use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, DocumentSnapshot,
    ImplementationPacketRef, LifecycleState, MetadataValue, ScenarioCoverage, Severity, state_of,
    validate_artifact, validate_reciprocal_relationships, validate_relationship_cycles,
    validate_relationships, validate_schema_links,
};

const MISSING_ARTIFACT_RULE: &str = "READINESS.PACKET.ARTIFACT_MISSING";
const LIFECYCLE_RULE: &str = "READINESS.PACKET.LIFECYCLE";
const COVERAGE_RULE: &str = "READINESS.SCENARIO.COVERAGE";
const BLOCKER_RULE: &str = "READINESS.PACKET.BLOCKER";
const PREDICATE_RULE: &str = "READINESS.PACKET.PREDICATE";
const DEPENDENCY_RULE: &str = "READINESS.DEPENDENCY.UNMET";
const CYCLE_RULE: &str = "READINESS.DEPENDENCY.CYCLE";
const EVIDENCE_RULE: &str = "READINESS.DEPENDENCY.EVIDENCE";

/// Reports the pure outcome of a Spec-Ready evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecReadyDecision {
    /// Every applicable readiness condition passed.
    Ready(SpecReadyResult),
    /// One or more applicable readiness conditions failed.
    NotReady(SpecReadyResult),
}

impl SpecReadyDecision {
    /// Returns the contained readiness result.
    #[must_use]
    pub const fn result(&self) -> &SpecReadyResult {
        match self {
            Self::Ready(result) | Self::NotReady(result) => result,
        }
    }

    /// Returns whether every readiness condition passed.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Adds source diagnostics while preserving deterministic result ordering.
    #[must_use]
    pub fn with_diagnostics(self, diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        let result = match self {
            Self::Ready(result) | Self::NotReady(result) => result,
        };
        let mut all = result.diagnostics;
        all.extend(diagnostics);
        let result = SpecReadyResult::new(result.packet, result.scope, all);
        if result.is_ready() { Self::Ready(result) } else { Self::NotReady(result) }
    }
}

/// Contains the deterministic scope and findings for one packet evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecReadyResult {
    packet: ImplementationPacketRef,
    scope: Vec<ArtifactPath>,
    diagnostics: Vec<Diagnostic>,
}

impl SpecReadyResult {
    fn new(
        packet: ImplementationPacketRef,
        mut scope: Vec<ArtifactPath>,
        mut diagnostics: Vec<Diagnostic>,
    ) -> Self {
        scope.sort();
        scope.dedup();
        diagnostics.sort_by(|left, right| {
            left.path()
                .cmp(right.path())
                .then_with(|| left.rule_id().cmp(right.rule_id()))
                .then_with(|| left.location().cmp(&right.location()))
                .then_with(|| left.message().cmp(right.message()))
        });
        Self { packet, scope, diagnostics }
    }

    /// Returns the evaluated packet reference.
    #[must_use]
    pub const fn packet(&self) -> &ImplementationPacketRef {
        &self.packet
    }

    /// Returns included packet, supporting, and dependency paths in stable order.
    #[must_use]
    pub fn scope(&self) -> &[ArtifactPath] {
        &self.scope
    }

    /// Returns every applicable unmet-condition diagnostic in stable order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether this result contains no unmet conditions.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Adds diagnostics from a normalized source boundary.
    #[must_use]
    pub fn with_diagnostics(self, diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        let mut all = self.diagnostics;
        all.extend(diagnostics);
        Self::new(self.packet, self.scope, all)
    }
}

/// Evaluates one implementation packet against the pure Spec-Ready predicate.
///
/// The evaluator reads only the supplied immutable snapshots. Missing content,
/// malformed content, lifecycle mismatches, and dependency failures become
/// deterministic diagnostics rather than operational I/O errors.
///
/// Covers: REQ-008 FR-001 through FR-010 and FR-012 through FR-013.
#[must_use]
pub fn evaluate_spec_ready(
    packet: &ImplementationPacketRef,
    snapshots: &[ArtifactSnapshot],
) -> SpecReadyDecision {
    let owned = snapshots.to_vec();
    let index = SnapshotIndex::new(&owned);
    let mut evaluation = Evaluation::new(packet.clone(), &index);
    evaluation.evaluate_root();
    evaluation.finish()
}

struct SnapshotIndex<'a> {
    by_path: BTreeMap<ArtifactPath, &'a ArtifactSnapshot>,
    by_id: BTreeMap<String, Vec<&'a ArtifactSnapshot>>,
}

impl<'a> SnapshotIndex<'a> {
    fn new(snapshots: &'a [ArtifactSnapshot]) -> Self {
        let mut by_path = BTreeMap::new();
        let mut by_id: BTreeMap<String, Vec<&'a ArtifactSnapshot>> = BTreeMap::new();
        for snapshot in snapshots {
            by_path.insert(snapshot.path().clone(), snapshot);
            if let Some(id) = snapshot.id() {
                by_id.entry(id.as_str().to_owned()).or_default().push(snapshot);
            }
        }
        for candidates in by_id.values_mut() {
            candidates.sort_by(|left, right| left.path().cmp(right.path()));
        }
        Self { by_path, by_id }
    }

    fn by_id(&self, id: &str, kind: ArtifactKind) -> Option<&'a ArtifactSnapshot> {
        self.by_id.get(id).and_then(|candidates| {
            candidates.iter().copied().find(|snapshot| snapshot.kind() == kind)
        })
    }
}

struct Evaluation<'a> {
    packet: ImplementationPacketRef,
    index: &'a SnapshotIndex<'a>,
    scope: BTreeSet<ArtifactPath>,
    relevant: BTreeSet<ArtifactPath>,
    diagnostics: Vec<Diagnostic>,
    visited_dependencies: BTreeSet<String>,
    visiting_dependencies: Vec<String>,
    cycles: BTreeSet<String>,
}

impl<'a> Evaluation<'a> {
    fn new(packet: ImplementationPacketRef, index: &'a SnapshotIndex<'a>) -> Self {
        Self {
            packet,
            index,
            scope: BTreeSet::new(),
            relevant: BTreeSet::new(),
            diagnostics: Vec::new(),
            visited_dependencies: BTreeSet::new(),
            visiting_dependencies: Vec::new(),
            cycles: BTreeSet::new(),
        }
    }

    fn evaluate_root(&mut self) {
        let root_paths = self.packet.colocated_paths();
        let mut root_snapshots = Vec::new();
        for path in &root_paths {
            self.scope.insert(path.clone());
            self.relevant.insert(path.clone());
            match self.index.by_path.get(path) {
                Some(snapshot) => root_snapshots.push(*snapshot),
                None => self.diagnostics.push(missing_artifact(path)),
            }
        }

        self.add_supporting_context(&root_snapshots);
        self.add_parent_context(&root_snapshots);
        self.add_scope_validations();
        self.evaluate_root_predicates(&root_snapshots);
        self.evaluate_dependencies(&root_snapshots);
    }

    fn add_supporting_context(&mut self, roots: &[&ArtifactSnapshot]) {
        let mut queue = roots.to_vec();
        let mut seen = BTreeSet::new();
        while let Some(snapshot) = queue.pop() {
            if !seen.insert(snapshot.path().clone()) {
                continue;
            }
            for reference in supporting_references(snapshot) {
                let Some(target) = self.index.by_id.get(&reference).and_then(|items| {
                    items.iter().copied().find(|candidate| {
                        matches!(
                            candidate.kind(),
                            ArtifactKind::Adr | ArtifactKind::Database | ArtifactKind::Table
                        )
                    })
                }) else {
                    continue;
                };
                if self.scope.insert(target.path().clone()) {
                    self.relevant.insert(target.path().clone());
                    queue.push(target);
                }
            }
        }
    }

    fn add_parent_context(&mut self, roots: &[&ArtifactSnapshot]) {
        let Some(story) =
            roots.iter().copied().find(|snapshot| snapshot.kind() == ArtifactKind::UserStory)
        else {
            return;
        };
        for (field, kind) in [("parent", ArtifactKind::Prd), ("epic", ArtifactKind::Epic)] {
            let Some(reference) = scalar_metadata(story, field) else {
                self.diagnostics.push(predicate_diagnostic(
                    story.path(),
                    format!("packet story is missing `{field}` prerequisite reference"),
                ));
                continue;
            };
            let Some(target) = self.index.by_id(reference, kind) else {
                self.diagnostics.push(predicate_diagnostic(
                    story.path(),
                    format!("packet prerequisite `{reference}` cannot be resolved"),
                ));
                continue;
            };
            self.relevant.insert(target.path().clone());
            if state_of(target) != Some(LifecycleState::Approved) {
                self.diagnostics.push(lifecycle_diagnostic(target, LifecycleState::Approved));
            }
        }
    }

    fn add_scope_validations(&mut self) {
        let relevant = self
            .relevant
            .iter()
            .filter_map(|path| self.index.by_path.get(path).copied())
            .collect::<Vec<_>>();
        for snapshot in &relevant {
            self.diagnostics.extend(validate_artifact(snapshot));
        }
        let all =
            self.index.by_path.values().map(|snapshot| (*snapshot).clone()).collect::<Vec<_>>();
        self.diagnostics.extend(
            validate_relationships(&all)
                .into_iter()
                .filter(|diagnostic| self.relevant.contains(diagnostic.path())),
        );
        self.diagnostics.extend(
            validate_reciprocal_relationships(&all)
                .into_iter()
                .filter(|diagnostic| self.relevant.contains(diagnostic.path())),
        );
        self.diagnostics.extend(
            validate_relationship_cycles(&all)
                .into_iter()
                .filter(|diagnostic| self.relevant.contains(diagnostic.path())),
        );
        self.diagnostics.extend(
            validate_schema_links(&all)
                .into_iter()
                .filter(|diagnostic| self.relevant.contains(diagnostic.path())),
        );
    }

    fn evaluate_root_predicates(&mut self, roots: &[&ArtifactSnapshot]) {
        self.evaluate_root_artifacts();
        for snapshot in roots {
            self.check_blockers(snapshot);
        }
        self.evaluate_supporting_lifecycle();
    }

    fn evaluate_root_artifacts(&mut self) {
        for path in self.packet.colocated_paths() {
            let Some(snapshot) = self.index.by_path.get(&path).copied() else { continue };
            if state_of(snapshot) != Some(LifecycleState::Approved) {
                self.diagnostics.push(lifecycle_diagnostic(snapshot, LifecycleState::Approved));
            }
            match snapshot.kind() {
                ArtifactKind::Gherkin => self.check_coverage(snapshot),
                ArtifactKind::Task => self.check_actionable_tasks(snapshot),
                ArtifactKind::Design => self.check_design_contract(snapshot),
                _ => {}
            }
        }
    }

    fn evaluate_supporting_lifecycle(&mut self) {
        let root_paths = self.packet.colocated_paths().into_iter().collect::<BTreeSet<_>>();
        for path in &self.scope {
            if root_paths.contains(path) {
                continue;
            }
            let Some(snapshot) = self.index.by_path.get(path).copied() else { continue };
            if matches!(
                snapshot.kind(),
                ArtifactKind::Adr | ArtifactKind::Database | ArtifactKind::Table
            ) && state_of(snapshot) != Some(LifecycleState::Approved)
            {
                self.diagnostics.push(lifecycle_diagnostic(snapshot, LifecycleState::Approved));
            }
        }
    }

    fn check_coverage(&mut self, snapshot: &ArtifactSnapshot) {
        let Some(feature) = snapshot.document().feature() else { return };
        for category in ScenarioCoverage::ALL {
            if !feature.coverage().contains(&category) {
                self.diagnostics.push(Diagnostic::new(
                    snapshot.path().clone(),
                    None,
                    COVERAGE_RULE,
                    Severity::Error,
                    format!("scenario coverage is missing `{}` behavior", category.prefix()),
                    format!("add a scenario whose name begins with `{}`", category.prefix()),
                ));
            }
        }
    }

    fn check_actionable_tasks(&mut self, snapshot: &ArtifactSnapshot) {
        let lines = snapshot.document().lines();
        for marker in ["RED", "GREEN"] {
            if !lines.iter().any(|line| line.text().contains(marker)) {
                self.diagnostics.push(Diagnostic::new(
                    snapshot.path().clone(),
                    None,
                    PREDICATE_RULE,
                    Severity::Error,
                    format!("tasks do not contain actionable {marker} test work"),
                    format!("add an explicit {marker} test task"),
                ));
            }
        }
    }

    fn check_design_contract(&mut self, snapshot: &ArtifactSnapshot) {
        let text = snapshot
            .document()
            .lines()
            .iter()
            .map(|line| line.text().to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("\n");
        for term in ["interfaces and contracts", "data and state flow", "errors"] {
            if !text.contains(term) {
                self.diagnostics.push(Diagnostic::new(
                    snapshot.path().clone(),
                    None,
                    PREDICATE_RULE,
                    Severity::Error,
                    format!("design does not contain an explicit `{term}` contract"),
                    format!("document `{term}` in the design"),
                ));
            }
        }
    }

    fn check_blockers(&mut self, snapshot: &ArtifactSnapshot) {
        let Some(value) = snapshot.metadata().get("blockers") else { return };
        for blocker in metadata_strings(value) {
            if !blocker.is_empty() {
                self.diagnostics.push(Diagnostic::new(
                    snapshot.path().clone(),
                    None,
                    BLOCKER_RULE,
                    Severity::Error,
                    format!("packet has unresolved blocker `{blocker}`"),
                    "resolve the blocker before evaluating the packet as Spec-Ready",
                ));
            }
        }
    }

    fn evaluate_dependencies(&mut self, roots: &[&ArtifactSnapshot]) {
        let Some(story) =
            roots.iter().copied().find(|snapshot| snapshot.kind() == ArtifactKind::UserStory)
        else {
            return;
        };
        for dependency in metadata_strings_option(story, "depends_on") {
            self.evaluate_dependency(&dependency, story.path());
        }
    }

    fn evaluate_dependency(&mut self, dependency_id: &str, source_path: &ArtifactPath) {
        let Some(story) = self.index.by_id(dependency_id, ArtifactKind::UserStory) else {
            self.diagnostics.push(Diagnostic::new(
                source_path.clone(),
                None,
                DEPENDENCY_RULE,
                Severity::Error,
                format!("dependency `{dependency_id}` cannot be resolved"),
                "add the dependency packet or correct the dependency identifier",
            ));
            return;
        };
        let root = packet_root(story.path());
        let key = root.as_str().to_owned();
        if let Some(position) = self.visiting_dependencies.iter().position(|item| item == &key) {
            let mut cycle =
                self.visiting_dependencies.get(position..).map_or_else(Vec::new, ToOwned::to_owned);
            cycle.push(key);
            let identity = cycle.join(" -> ");
            if self.cycles.insert(identity.clone()) {
                self.diagnostics.push(Diagnostic::new(
                    source_path.clone(),
                    None,
                    CYCLE_RULE,
                    Severity::Error,
                    format!("dependency cycle detected: `{identity}`"),
                    "break the recursive feature dependency cycle",
                ));
            }
            return;
        }
        if !self.visited_dependencies.insert(key.clone()) {
            return;
        }
        self.visiting_dependencies.push(key);
        let dependency_paths = packet_paths(&root);
        for path in &dependency_paths {
            self.scope.insert(path.clone());
            self.relevant.insert(path.clone());
            let Some(snapshot) = self.index.by_path.get(path).copied() else {
                self.diagnostics.push(Diagnostic::new(
                    path.clone(),
                    None,
                    DEPENDENCY_RULE,
                    Severity::Error,
                    format!("implemented dependency `{dependency_id}` is missing packet artifact"),
                    "complete and verify every dependency packet artifact",
                ));
                continue;
            };
            if state_of(snapshot) != Some(LifecycleState::Implemented) {
                self.diagnostics.push(Diagnostic::new(
                    path.clone(),
                    None,
                    DEPENDENCY_RULE,
                    Severity::Error,
                    format!("dependency artifact `{}` is not implemented", snapshot.path()),
                    "implement and verify the dependency packet before readiness evaluation",
                ));
            }
            if snapshot.kind() == ArtifactKind::Task
                && !has_verification_evidence(snapshot.document())
            {
                self.diagnostics.push(Diagnostic::new(
                    path.clone(),
                    None,
                    EVIDENCE_RULE,
                    Severity::Error,
                    format!("dependency `{dependency_id}` lacks verification evidence"),
                    "record observed implementation and verification evidence in tasks.md",
                ));
            }
        }
        for nested in metadata_strings_option(story, "depends_on") {
            self.evaluate_dependency(&nested, story.path());
        }
        let _ = self.visiting_dependencies.pop();
    }

    fn finish(self) -> SpecReadyDecision {
        let result =
            SpecReadyResult::new(self.packet, self.scope.into_iter().collect(), self.diagnostics);
        if result.is_ready() {
            SpecReadyDecision::Ready(result)
        } else {
            SpecReadyDecision::NotReady(result)
        }
    }
}

fn packet_root(path: &ArtifactPath) -> ArtifactPath {
    let root = path.as_str().rsplit_once('/').map_or(path.as_str(), |(root, _)| root);
    ArtifactPath::try_new(root.to_owned()).unwrap_or_else(|_| path.clone())
}

fn packet_paths(root: &ArtifactPath) -> Vec<ArtifactPath> {
    ["user-story.md", "scenarios.feature", "requirements.md", "design.md", "tasks.md"]
        .into_iter()
        .filter_map(|file| ArtifactPath::try_new(format!("{}/{file}", root.as_str())).ok())
        .collect()
}

fn supporting_references(snapshot: &ArtifactSnapshot) -> Vec<String> {
    ["requires", "related"]
        .into_iter()
        .flat_map(|field| metadata_strings_option(snapshot, field))
        .collect()
}

fn metadata_strings_option(snapshot: &ArtifactSnapshot, field: &str) -> Vec<String> {
    snapshot.metadata().get(field).map_or_else(Vec::new, metadata_strings)
}

fn metadata_strings(value: &MetadataValue) -> Vec<String> {
    match value {
        MetadataValue::Scalar(value) => vec![value.clone()],
        MetadataValue::Sequence(values) => values.clone(),
        MetadataValue::Null | MetadataValue::Mapping => Vec::new(),
    }
}

fn scalar_metadata<'a>(snapshot: &'a ArtifactSnapshot, field: &str) -> Option<&'a str> {
    match snapshot.metadata().get(field) {
        Some(MetadataValue::Scalar(value)) => Some(value),
        _ => None,
    }
}

fn missing_artifact(path: &ArtifactPath) -> Diagnostic {
    Diagnostic::new(
        path.clone(),
        None,
        MISSING_ARTIFACT_RULE,
        Severity::Error,
        format!("required packet artifact `{path}` is missing"),
        "create the colocated implementation packet artifact",
    )
}

fn lifecycle_diagnostic(snapshot: &ArtifactSnapshot, expected: LifecycleState) -> Diagnostic {
    let actual = state_of(snapshot).map_or("missing", LifecycleState::as_str);
    Diagnostic::new(
        snapshot.path().clone(),
        None,
        LIFECYCLE_RULE,
        Severity::Error,
        format!(
            "artifact lifecycle state is `{actual}` but Spec-Ready requires `{}`",
            expected.as_str()
        ),
        format!("promote the artifact to `{}` before readiness evaluation", expected.as_str()),
    )
}

fn predicate_diagnostic(path: &ArtifactPath, message: String) -> Diagnostic {
    Diagnostic::new(
        path.clone(),
        None,
        PREDICATE_RULE,
        Severity::Error,
        message,
        "complete the normative Spec-Ready prerequisite",
    )
}

fn has_verification_evidence(document: &DocumentSnapshot) -> bool {
    document.lines().iter().any(|line| {
        let text = line.text().to_ascii_lowercase();
        text.contains("observed verification evidence")
            || text.contains("verification evidence") && text.contains("passed")
    })
}
