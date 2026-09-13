use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{
    ArtifactId, ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, LifecycleState,
    MetadataValue, PromotionActor, PromotionFacts, PromotionPlan, PromotionRequest,
    PromotionTimestamp, Severity, decide_promotion, state_of,
};

/// Identifies one implementation packet by its repository-relative directory.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImplementationPacketRef {
    path: ArtifactPath,
}

impl ImplementationPacketRef {
    /// Creates an implementation packet reference.
    ///
    /// # Errors
    ///
    /// Returns [`PacketReferenceError::Invalid`] when the value is not a directory
    /// below the repository `specs/` tree.
    pub fn try_new(value: impl Into<String>) -> Result<Self, PacketReferenceError> {
        let path = ArtifactPath::try_new(value).map_err(|error| {
            PacketReferenceError::Invalid(format!("implementation packet path is invalid: {error}"))
        })?;
        let directory = path.as_str().strip_prefix("specs/");
        let is_numbered_packet_directory = directory.is_some_and(|value| {
            value.len() > 4
                && value.as_bytes().get(3) == Some(&b'-')
                && value
                    .as_bytes()
                    .get(0..3)
                    .is_some_and(|number| number.iter().all(u8::is_ascii_digit))
                && !value.contains('/')
        });
        let is_specs_directory =
            is_numbered_packet_directory && ArtifactKind::from_path(path.as_str()).is_none();
        if !is_specs_directory {
            return Err(PacketReferenceError::Invalid(
                "implementation packet must be a repository-relative directory below specs/"
                    .to_string(),
            ));
        }
        Ok(Self { path })
    }

    /// Returns the packet directory path.
    #[must_use]
    pub const fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the five recognized colocated packet artifact paths.
    #[must_use]
    pub fn colocated_paths(&self) -> Vec<ArtifactPath> {
        ["user-story.md", "scenarios.feature", "requirements.md", "design.md", "tasks.md"]
            .into_iter()
            .filter_map(|file| ArtifactPath::try_new(format!("{}/{file}", self.path)).ok())
            .collect()
    }
}

/// Explains why an implementation packet reference was rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketReferenceError {
    /// The supplied value is not a valid packet directory.
    Invalid(String),
}

impl Display for PacketReferenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl Error for PacketReferenceError {}

/// Identifies why an artifact belongs to a packet promotion set.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PacketParticipantRole {
    /// The artifact is one of the five files colocated with the packet.
    Colocated,
    /// The artifact is an implementation-packet supporting artifact.
    Supporting,
}

/// Describes one included packet artifact without carrying source-format data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketParticipant {
    path: ArtifactPath,
    kind: ArtifactKind,
    id: Option<ArtifactId>,
    role: PacketParticipantRole,
}

impl PacketParticipant {
    fn from_snapshot(snapshot: &ArtifactSnapshot, role: PacketParticipantRole) -> Self {
        Self {
            path: snapshot.path().clone(),
            kind: snapshot.kind(),
            id: snapshot.id().cloned(),
            role,
        }
    }

    /// Returns the participant path.
    #[must_use]
    pub const fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the participant artifact kind.
    #[must_use]
    pub const fn kind(&self) -> ArtifactKind {
        self.kind
    }

    /// Returns the participant identifier, when the artifact has one.
    #[must_use]
    pub fn id(&self) -> Option<&ArtifactId> {
        self.id.as_ref()
    }

    /// Returns the participant role.
    #[must_use]
    pub const fn role(&self) -> PacketParticipantRole {
        self.role
    }
}

/// Supplies prerequisite facts for packet participants, keyed by ID or artifact kind.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PacketPromotionFacts {
    by_id: BTreeMap<String, PromotionFacts>,
    by_kind: BTreeMap<ArtifactKind, PromotionFacts>,
}

impl PacketPromotionFacts {
    /// Adds or replaces facts for one artifact identifier.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>, facts: PromotionFacts) -> Self {
        self.by_id.insert(id.into(), facts);
        self
    }

    /// Adds or replaces facts for an artifact kind.
    #[must_use]
    pub fn with_kind(mut self, kind: ArtifactKind, facts: PromotionFacts) -> Self {
        self.by_kind.insert(kind, facts);
        self
    }

    /// Inserts facts for one artifact identifier.
    pub fn insert_id(&mut self, id: impl Into<String>, facts: PromotionFacts) {
        self.by_id.insert(id.into(), facts);
    }

    /// Inserts facts for an artifact kind.
    pub fn insert_kind(&mut self, kind: ArtifactKind, facts: PromotionFacts) {
        self.by_kind.insert(kind, facts);
    }

    /// Returns facts previously stored for one identifier.
    #[must_use]
    pub fn facts_for_id(&self, id: &str) -> Option<&PromotionFacts> {
        self.by_id.get(id)
    }

    /// Returns facts previously stored for one artifact kind.
    #[must_use]
    pub fn facts_for_kind(&self, kind: ArtifactKind) -> Option<&PromotionFacts> {
        self.by_kind.get(&kind)
    }

    /// Returns the most-specific facts for a participant and target state.
    #[must_use]
    pub fn facts_for(&self, snapshot: &ArtifactSnapshot, target: LifecycleState) -> PromotionFacts {
        if let Some(id) = snapshot.id().and_then(|id| self.by_id.get(id.as_str())) {
            return id.clone();
        }
        if let Some(kind) = self.by_kind.get(&snapshot.kind()) {
            return kind.clone();
        }
        missing_target_facts(target)
    }
}

fn missing_target_facts(target: LifecycleState) -> PromotionFacts {
    match target {
        LifecycleState::Approved => PromotionFacts::passing().with_human_approval_confirmed(false),
        LifecycleState::Implemented => {
            PromotionFacts::passing().with_implementation_evidence(false)
        }
        LifecycleState::Archived => PromotionFacts::passing()
            .with_archive_location(false)
            .with_closed_release_conditions(false),
        LifecycleState::Draft | LifecycleState::InReview | LifecycleState::Superseded => {
            PromotionFacts::passing()
        }
    }
}

/// One packet participant's pure promotion step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketPromotionStep {
    /// The participant has an accepted lifecycle transition.
    Advance(PromotionPlan),
    /// The participant has no applicable next lifecycle transition.
    Skip(PacketParticipant),
}

/// Accepted packet plans containing all mutations and skipped participants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketPromotionPlan {
    advanced: Vec<PromotionPlan>,
    skipped: Vec<PacketParticipant>,
}

impl PacketPromotionPlan {
    fn new(advanced: Vec<PromotionPlan>, skipped: Vec<PacketParticipant>) -> Self {
        Self { advanced, skipped }
    }

    /// Returns accepted artifact promotions in deterministic path order.
    #[must_use]
    pub fn advanced(&self) -> &[PromotionPlan] {
        &self.advanced
    }

    /// Returns included artifacts that were intentionally skipped.
    #[must_use]
    pub fn skipped(&self) -> &[PacketParticipant] {
        &self.skipped
    }

    /// Returns whether this plan performs no writes.
    #[must_use]
    pub const fn is_noop(&self) -> bool {
        self.advanced.is_empty()
    }
}

/// Represents the pure packet promotion decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketPromotionDecision {
    /// All advanceable participants passed and may be committed atomically.
    Accepted(PacketPromotionPlan),
    /// One or more selected participants failed preflight.
    Rejected(Vec<Diagnostic>),
}

/// Plans one lifecycle step for every included implementation-packet artifact.
///
/// This function performs no I/O and never mutates the supplied snapshots.
#[must_use]
pub fn plan_packet_promotion(
    packet: &ImplementationPacketRef,
    snapshots: &[ArtifactSnapshot],
    facts: &PacketPromotionFacts,
    actor: &PromotionActor,
    promoted_at: PromotionTimestamp,
) -> PacketPromotionDecision {
    let participants = included_participants(packet, snapshots);
    let snapshots_by_path = snapshots
        .iter()
        .map(|snapshot| (snapshot.path().clone(), snapshot))
        .collect::<BTreeMap<_, _>>();
    let mut advanced = Vec::new();
    let mut skipped = Vec::new();
    let mut diagnostics = Vec::new();

    for participant in participants {
        let Some(snapshot) = snapshots_by_path.get(participant.path()) else { continue };
        let Some(source) = state_of(snapshot) else {
            diagnostics.push(packet_diagnostic(
                snapshot,
                "STATE_MISSING",
                "artifact lifecycle state is missing or unsupported",
                "set the artifact status to a supported lifecycle value",
            ));
            continue;
        };
        let Some(target) = next_lifecycle_state(source) else {
            skipped.push(participant);
            continue;
        };
        let request = PromotionRequest::new(target, actor.clone());
        let participant_facts = facts.facts_for(snapshot, target);
        match decide_promotion(snapshot, &participant_facts, &request, promoted_at) {
            crate::PromotionDecision::Accepted(plan) => advanced.push(plan),
            crate::PromotionDecision::Idempotent => skipped.push(participant),
            crate::PromotionDecision::Rejected(mut findings) => diagnostics.append(&mut findings),
        }
    }

    if !diagnostics.is_empty() {
        diagnostics.sort_by(|left, right| {
            left.path().cmp(right.path()).then_with(|| left.stable_cmp(right))
        });
        return PacketPromotionDecision::Rejected(diagnostics);
    }
    advanced.sort_by(|left, right| left.path().cmp(right.path()));
    skipped.sort_by(|left, right| left.path().cmp(right.path()));
    PacketPromotionDecision::Accepted(PacketPromotionPlan::new(advanced, skipped))
}

fn included_participants(
    packet: &ImplementationPacketRef,
    snapshots: &[ArtifactSnapshot],
) -> Vec<PacketParticipant> {
    let by_path = snapshots
        .iter()
        .map(|snapshot| (snapshot.path().clone(), snapshot))
        .collect::<BTreeMap<_, _>>();
    let mut by_id: BTreeMap<&str, Vec<&ArtifactSnapshot>> = BTreeMap::new();
    for snapshot in snapshots {
        if let Some(id) = snapshot.id() {
            by_id.entry(id.as_str()).or_default().push(snapshot);
        }
    }

    let colocated_paths = packet.colocated_paths().into_iter().collect::<BTreeSet<_>>();
    let mut selected = BTreeMap::new();
    let mut queue = VecDeque::new();
    for path in colocated_paths {
        if let Some(snapshot) = by_path.get(&path) {
            selected.insert(
                path.clone(),
                PacketParticipant::from_snapshot(snapshot, PacketParticipantRole::Colocated),
            );
            queue.push_back(path);
        }
    }

    while let Some(path) = queue.pop_front() {
        let Some(snapshot) = by_path.get(&path) else { continue };
        for reference in relationship_references(snapshot) {
            let Some(targets) = by_id.get(reference.as_str()) else { continue };
            for target in targets {
                if target.kind() != ArtifactKind::Adr || selected.contains_key(target.path()) {
                    continue;
                }
                selected.insert(
                    target.path().clone(),
                    PacketParticipant::from_snapshot(target, PacketParticipantRole::Supporting),
                );
                queue.push_back(target.path().clone());
            }
        }
    }
    selected.into_values().collect()
}

fn relationship_references(snapshot: &ArtifactSnapshot) -> Vec<String> {
    ["requires", "depends_on", "related"]
        .into_iter()
        .filter_map(|field| snapshot.metadata().get(field))
        .flat_map(metadata_strings)
        .filter(|value| !value.is_empty())
        .collect()
}

fn metadata_strings(value: &MetadataValue) -> Vec<String> {
    match value {
        MetadataValue::Scalar(value) => vec![value.clone()],
        MetadataValue::Sequence(values) => values.clone(),
        MetadataValue::Null | MetadataValue::Mapping => Vec::new(),
    }
}

/// Returns the strict next lifecycle state, when one exists.
#[must_use]
pub fn next_lifecycle_state(source: LifecycleState) -> Option<LifecycleState> {
    match source {
        LifecycleState::Draft => Some(LifecycleState::InReview),
        LifecycleState::InReview => Some(LifecycleState::Approved),
        LifecycleState::Approved => Some(LifecycleState::Implemented),
        LifecycleState::Implemented => Some(LifecycleState::Archived),
        LifecycleState::Archived | LifecycleState::Superseded => None,
    }
}

fn packet_diagnostic(
    snapshot: &ArtifactSnapshot,
    rule: &str,
    message: impl Into<String>,
    remediation: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        snapshot.path().clone(),
        None,
        format!("ARTIFACT.PACKET.{rule}"),
        Severity::Error,
        message,
        remediation,
    )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::{DocumentSnapshot, Metadata};

    fn path(value: &str) -> ArtifactPath {
        match ArtifactPath::try_new(value) {
            Ok(path) => path,
            Err(_) => std::process::abort(),
        }
    }

    fn actor() -> PromotionActor {
        match PromotionActor::try_new("agent-1") {
            Ok(actor) => actor,
            Err(_) => std::process::abort(),
        }
    }

    fn snapshot(
        path_value: &str,
        kind: ArtifactKind,
        id: Option<&str>,
        status: &str,
        related: &[&str],
    ) -> ArtifactSnapshot {
        let mut metadata = Metadata::new();
        metadata.insert_scalar("status", status);
        if let Some(id) = id {
            metadata.insert_scalar("id", id);
        }
        metadata.insert_sequence("related", related.iter().copied());
        ArtifactSnapshot::new(path(path_value), kind, metadata, DocumentSnapshot::empty())
    }

    /// Covers: REQ-007 packet reference and participant value contracts.
    #[test]
    fn validates_packet_references() {
        assert!(ImplementationPacketRef::try_new("").is_err());
        assert!(ImplementationPacketRef::try_new("specs/not-a-packet").is_err());
        assert!(ImplementationPacketRef::try_new("specs/007-packet/user-story.md").is_err());
        let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
            std::process::abort()
        };
        assert_eq!(packet.path().as_str(), "specs/007-packet");
        assert_eq!(packet.colocated_paths().len(), 5);
        assert_eq!(PacketReferenceError::Invalid("bad".to_string()).to_string(), "bad");
    }

    /// Covers: REQ-007 fact-key precedence and target defaults.
    #[test]
    fn matches_id_facts_before_kind_facts() {
        let story = snapshot(
            "specs/007-packet/user-story.md",
            ArtifactKind::UserStory,
            Some("US-007"),
            "draft",
            &[],
        );
        let id_facts = PromotionFacts::passing().with_human_approval_confirmed(false);
        let facts = PacketPromotionFacts::default()
            .with_kind(ArtifactKind::UserStory, PromotionFacts::passing())
            .with_id("US-007", id_facts.clone());
        assert_eq!(facts.facts_for(&story, LifecycleState::Approved), id_facts);

        let mut inserted = PacketPromotionFacts::default();
        inserted.insert_kind(ArtifactKind::UserStory, PromotionFacts::passing());
        inserted.insert_id("US-007", id_facts.clone());
        assert_eq!(inserted.facts_for(&story, LifecycleState::Approved), id_facts);
        assert_eq!(
            PacketPromotionFacts::default().facts_for(&story, LifecycleState::Approved),
            PromotionFacts::passing().with_human_approval_confirmed(false)
        );
        assert_eq!(
            PacketPromotionFacts::default().facts_for(&story, LifecycleState::Implemented),
            PromotionFacts::passing().with_implementation_evidence(false)
        );
        assert_eq!(
            PacketPromotionFacts::default().facts_for(&story, LifecycleState::Archived),
            PromotionFacts::passing()
                .with_archive_location(false)
                .with_closed_release_conditions(false)
        );
        assert_eq!(
            PacketPromotionFacts::default().facts_for(&story, LifecycleState::InReview),
            PromotionFacts::passing()
        );
        let _step = PacketPromotionStep::Skip(PacketParticipant::from_snapshot(
            &story,
            PacketParticipantRole::Colocated,
        ));
    }

    /// Covers: REQ-007 FR-002 through FR-005 — packet inclusion is bounded to colocated files and ADR support.
    #[test]
    fn includes_colocated_artifacts_and_referenced_adrs_but_excludes_parent_context() {
        let Ok(packet) =
            ImplementationPacketRef::try_new("specs/007-promote-implementation-packet")
        else {
            std::process::abort()
        };
        let snapshots = vec![
            snapshot(
                "specs/007-promote-implementation-packet/user-story.md",
                ArtifactKind::UserStory,
                Some("US-007"),
                "draft",
                &["ADR-007", "PRD-001"],
            ),
            snapshot(
                "specs/007-promote-implementation-packet/scenarios.feature",
                ArtifactKind::Gherkin,
                None,
                "draft",
                &[],
            ),
            snapshot(
                "specs/007-promote-implementation-packet/requirements.md",
                ArtifactKind::Requirements,
                Some("REQ-007"),
                "draft",
                &[],
            ),
            snapshot(
                "specs/007-promote-implementation-packet/design.md",
                ArtifactKind::Design,
                Some("DES-007"),
                "draft",
                &[],
            ),
            snapshot(
                "specs/007-promote-implementation-packet/tasks.md",
                ArtifactKind::Task,
                Some("TASK-007"),
                "draft",
                &[],
            ),
            snapshot("specs/adr/ADR-007.md", ArtifactKind::Adr, Some("ADR-007"), "draft", &[]),
            snapshot("specs/prds/PRD-001.md", ArtifactKind::Prd, Some("PRD-001"), "draft", &[]),
        ];

        let decision = plan_packet_promotion(
            &packet,
            &snapshots,
            &PacketPromotionFacts::default(),
            &actor(),
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PacketPromotionDecision::Accepted(plan) = decision else {
            assert!(
                matches!(decision, PacketPromotionDecision::Accepted(_)),
                "unexpected decision: {decision:?}"
            );
            return;
        };
        let advanced = plan.advanced();
        assert_eq!(advanced.len(), 6);
        assert!(!plan.is_noop());
        assert!(advanced.iter().any(|item| item.path().as_str() == "specs/adr/ADR-007.md"));
        assert!(!advanced.iter().any(|item| item.path().as_str() == "specs/prds/PRD-001.md"));
    }

    /// Covers: REQ-007 FR-006 and FR-008 — each artifact derives its own state and ID facts win.
    #[test]
    fn derives_each_artifact_next_state_and_matches_id_facts_before_kind_facts() {
        let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
            std::process::abort()
        };
        let snapshots = vec![
            snapshot(
                "specs/007-packet/user-story.md",
                ArtifactKind::UserStory,
                Some("US-007"),
                "draft",
                &[],
            ),
            snapshot(
                "specs/007-packet/requirements.md",
                ArtifactKind::Requirements,
                Some("REQ-007"),
                "in-review",
                &[],
            ),
        ];
        let facts = PacketPromotionFacts::default()
            .with_kind(
                ArtifactKind::Requirements,
                PromotionFacts::passing().with_human_approval_confirmed(false),
            )
            .with_id("REQ-007", PromotionFacts::passing());

        let decision = plan_packet_promotion(
            &packet,
            &snapshots,
            &facts,
            &actor(),
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PacketPromotionDecision::Accepted(plan) = decision else { std::process::abort() };
        assert_eq!(plan.advanced().len(), 2);
        assert!(plan.advanced().iter().any(|item| {
            item.path().as_str() == "specs/007-packet/requirements.md"
                && item.source() == LifecycleState::InReview
                && item.target() == LifecycleState::Approved
        }));
    }

    /// Covers: REQ-007 FR-009 — terminal participants are skipped, including an all-skipped no-op.
    #[test]
    fn reports_terminal_participants_as_skipped_and_all_skipped_is_successful_noop() {
        let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
            std::process::abort()
        };
        let snapshots = vec![snapshot(
            "specs/007-packet/tasks.md",
            ArtifactKind::Task,
            Some("TASK-007"),
            "archived",
            &[],
        )];

        let decision = plan_packet_promotion(
            &packet,
            &snapshots,
            &PacketPromotionFacts::default(),
            &actor(),
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PacketPromotionDecision::Accepted(plan) = decision else { std::process::abort() };
        assert!(plan.advanced().is_empty());
        assert_eq!(plan.skipped().len(), 1);
    }

    // Covers: REQ-007 quality requirements — packet planning is independent of snapshot order.
    proptest! {
        #[test]
        fn planning_is_deterministic_when_snapshots_are_reordered(reverse in any::<bool>()) {
            let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
                prop_assert!(false);
                return Ok(());
            };
            let mut snapshots = vec![
                snapshot("specs/007-packet/user-story.md", ArtifactKind::UserStory, Some("US-007"), "draft", &[]),
                snapshot("specs/007-packet/requirements.md", ArtifactKind::Requirements, Some("REQ-007"), "draft", &[]),
            ];
            let first = plan_packet_promotion(
                &packet,
                &snapshots,
                &PacketPromotionFacts::default(),
                &actor(),
                PromotionTimestamp::from_unix_seconds(42),
            );
            if reverse {
                snapshots.reverse();
            }
            let second = plan_packet_promotion(
                &packet,
                &snapshots,
                &PacketPromotionFacts::default(),
                &actor(),
                PromotionTimestamp::from_unix_seconds(42),
            );
            prop_assert_eq!(first, second);
        }
    }

    /// Covers: REQ-007 recursive support traversal and outside-context exclusion.
    #[test]
    fn recursively_includes_supporting_adrs_and_skips_other_kinds() {
        let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
            std::process::abort()
        };
        let mut story_metadata = Metadata::new();
        story_metadata.insert_scalar("status", "draft");
        story_metadata.insert_scalar("id", "US-007");
        story_metadata.insert_scalar("requires", "ADR-001");
        story_metadata.insert("depends_on", MetadataValue::Null);
        story_metadata.insert("related", MetadataValue::Mapping);
        let story = ArtifactSnapshot::new(
            path("specs/007-packet/user-story.md"),
            ArtifactKind::UserStory,
            story_metadata,
            DocumentSnapshot::empty(),
        );
        let adr_one = snapshot(
            "specs/adr/ADR-001.md",
            ArtifactKind::Adr,
            Some("ADR-001"),
            "draft",
            &["ADR-002", "REQ-999"],
        );
        let adr_two =
            snapshot("specs/adr/ADR-002.md", ArtifactKind::Adr, Some("ADR-002"), "archived", &[]);
        let outside = snapshot(
            "specs/other/requirements.md",
            ArtifactKind::Requirements,
            Some("REQ-999"),
            "draft",
            &[],
        );

        let decision = plan_packet_promotion(
            &packet,
            &[story, adr_one, adr_two, outside],
            &PacketPromotionFacts::default(),
            &actor(),
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PacketPromotionDecision::Accepted(plan) = decision else { std::process::abort() };
        assert_eq!(plan.advanced().len(), 2);
        assert_eq!(plan.skipped().len(), 1);
        let Some(skipped) = plan.skipped().first() else { std::process::abort() };
        assert_eq!(skipped.kind(), ArtifactKind::Adr);
        assert_eq!(skipped.id().map(ArtifactId::as_str), Some("ADR-002"));
        assert_eq!(skipped.role(), PacketParticipantRole::Supporting);
        assert_eq!(next_lifecycle_state(LifecycleState::Superseded), None);
    }

    /// Covers: REQ-007 FR-013 — missing lifecycle state is a deterministic rejection.
    #[test]
    fn rejects_an_included_artifact_without_lifecycle_state() {
        let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
            std::process::abort()
        };
        let mut metadata = Metadata::new();
        metadata.insert_scalar("id", "TASK-007");
        let snapshot = ArtifactSnapshot::new(
            path("specs/007-packet/tasks.md"),
            ArtifactKind::Task,
            metadata,
            DocumentSnapshot::empty(),
        );

        let decision = plan_packet_promotion(
            &packet,
            &[snapshot],
            &PacketPromotionFacts::default(),
            &actor(),
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PacketPromotionDecision::Rejected(diagnostics) = decision else {
            std::process::abort()
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str() == "ARTIFACT.PACKET.STATE_MISSING")
        );
    }

    /// Covers: REQ-007 FR-013 — packet diagnostics are complete and path ordered.
    #[test]
    fn aggregates_rejections_in_stable_path_order() {
        let Ok(packet) = ImplementationPacketRef::try_new("specs/007-packet") else {
            std::process::abort()
        };
        let snapshots = vec![
            snapshot(
                "specs/007-packet/design.md",
                ArtifactKind::Design,
                Some("DES-002"),
                "approved",
                &[],
            ),
            snapshot(
                "specs/007-packet/requirements.md",
                ArtifactKind::Requirements,
                Some("REQ-001"),
                "approved",
                &[],
            ),
        ];
        let facts = PacketPromotionFacts::default()
            .with_kind(
                ArtifactKind::Design,
                PromotionFacts::passing().with_blockers(vec!["OBS-001".to_string()]),
            )
            .with_kind(
                ArtifactKind::Requirements,
                PromotionFacts::passing().with_implementation_evidence(false),
            );

        let decision = plan_packet_promotion(
            &packet,
            &snapshots,
            &facts,
            &actor(),
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PacketPromotionDecision::Rejected(diagnostics) = decision else {
            std::process::abort()
        };
        assert_eq!(diagnostics.len(), 2);
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.rule_id().as_str().ends_with("BLOCKER_UNRESOLVED")
            })
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.rule_id().as_str().ends_with("EVIDENCE_MISSING") })
        );
        let Some(first) = diagnostics.first() else { std::process::abort() };
        let Some(second) = diagnostics.get(1) else { std::process::abort() };
        assert!(first.path().as_str() < second.path().as_str());
    }
}
