use std::collections::BTreeMap;

use domain::{
    ArtifactSnapshot, Diagnostic, ImplementationPacketRef, PacketParticipant,
    PacketPromotionDecision, PacketPromotionFacts, PacketPromotionPlan, PromotionActor,
    PromotionFacts, PromotionPlan, PromotionTimestamp, next_lifecycle_state, plan_packet_promotion,
    validate_artifact, validate_reciprocal_relationships, validate_relationship_cycles,
    validate_relationships,
};

use crate::{
    ArtifactIdentitySource, ArtifactPacketPromotionStore, BatchPromotionEntry, Clock,
    PromotionError,
};

/// Requests one derived lifecycle step for an implementation packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotePacketCommand {
    packet: ImplementationPacketRef,
    actor: PromotionActor,
    facts: PacketPromotionFacts,
}

impl PromotePacketCommand {
    /// Creates a packet promotion command without a packet-level target state.
    #[must_use]
    pub fn new(packet: ImplementationPacketRef, actor: PromotionActor) -> Self {
        Self { packet, actor, facts: PacketPromotionFacts::default() }
    }

    /// Supplies per-artifact prerequisite facts keyed by ID or artifact kind.
    #[must_use]
    pub fn with_facts(mut self, facts: PacketPromotionFacts) -> Self {
        self.facts = facts;
        self
    }

    /// Supplies facts for one artifact identifier.
    #[must_use]
    pub fn with_facts_for_id(mut self, id: impl Into<String>, facts: PromotionFacts) -> Self {
        self.facts.insert_id(id, facts);
        self
    }

    /// Supplies facts for one artifact kind.
    #[must_use]
    pub fn with_facts_for_kind(
        mut self,
        kind: domain::ArtifactKind,
        facts: PromotionFacts,
    ) -> Self {
        self.facts.insert_kind(kind, facts);
        self
    }

    /// Returns the requested implementation packet.
    #[must_use]
    pub const fn packet(&self) -> &ImplementationPacketRef {
        &self.packet
    }

    /// Returns the requesting actor.
    #[must_use]
    pub const fn actor(&self) -> &PromotionActor {
        &self.actor
    }

    /// Returns supplied per-artifact facts.
    #[must_use]
    pub const fn facts(&self) -> &PacketPromotionFacts {
        &self.facts
    }
}

/// Reports every advancing and skipped participant after a successful packet request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketPromotionResult {
    advanced: Vec<PromotionPlan>,
    skipped: Vec<PacketParticipant>,
}

impl PacketPromotionResult {
    fn from_plan(plan: &PacketPromotionPlan) -> Self {
        Self { advanced: plan.advanced().to_vec(), skipped: plan.skipped().to_vec() }
    }

    /// Returns advancing participant plans in deterministic path order.
    #[must_use]
    pub fn advanced(&self) -> &[PromotionPlan] {
        &self.advanced
    }

    /// Returns skipped participants in deterministic path order.
    #[must_use]
    pub fn skipped(&self) -> &[PacketParticipant] {
        &self.skipped
    }
}

/// Reports the observable result of a packet promotion request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketPromotionOutcome {
    /// The packet plan was accepted and its advancing participants were committed.
    Promoted(PacketPromotionResult),
    /// The packet plan was rejected before mutation with complete diagnostics.
    Rejected(Vec<Diagnostic>),
}

/// Orchestrates one atomic implementation-packet promotion through application ports.
pub struct PacketPromoter<S, P, C> {
    source: S,
    store: P,
    clock: C,
}

impl<S, P, C> PacketPromoter<S, P, C>
where
    S: ArtifactIdentitySource,
    P: ArtifactPacketPromotionStore,
    C: Clock,
{
    /// Creates a packet promoter from repository discovery, batch storage, and a clock.
    #[must_use]
    pub const fn new(source: S, store: P, clock: C) -> Self {
        Self { source, store, clock }
    }

    /// Promotes every eligible included participant one derived lifecycle step.
    ///
    /// # Errors
    ///
    /// Returns typed operational errors for discovery, source loading, clock, conflict, or
    /// batch-write failures. Prerequisite failures are returned as a rejected outcome.
    pub fn promote(
        &mut self,
        command: &PromotePacketCommand,
    ) -> Result<PacketPromotionOutcome, PromotionError> {
        let candidates = self.source.discover_identities()?;
        let packet_paths = command.packet().colocated_paths();
        let mut source_diagnostics = candidates
            .iter()
            .filter(|candidate| packet_paths.iter().any(|path| path == candidate.path()))
            .filter_map(|candidate| candidate.diagnostic().cloned())
            .collect::<Vec<_>>();
        source_diagnostics.sort_by(|left, right| {
            left.path()
                .cmp(right.path())
                .then_with(|| left.rule_id().cmp(right.rule_id()))
                .then_with(|| left.message().cmp(right.message()))
        });
        if !source_diagnostics.is_empty() {
            return Ok(PacketPromotionOutcome::Rejected(source_diagnostics));
        }
        let snapshots = candidates
            .iter()
            .filter_map(|candidate| candidate.snapshot().cloned())
            .collect::<Vec<_>>();
        let facts = packet_facts(&snapshots, command);
        let preliminary = plan_packet_promotion(
            command.packet(),
            &snapshots,
            &facts,
            command.actor(),
            PromotionTimestamp::from_unix_seconds(0),
        );
        let PacketPromotionDecision::Accepted(preliminary_plan) = preliminary else {
            let PacketPromotionDecision::Rejected(diagnostics) = preliminary else {
                return Err(PromotionError::InconsistentDecision);
            };
            return Ok(PacketPromotionOutcome::Rejected(diagnostics));
        };
        if preliminary_plan.is_noop() {
            return Ok(PacketPromotionOutcome::Promoted(PacketPromotionResult::from_plan(
                &preliminary_plan,
            )));
        }

        let promoted_at = self.clock.now()?;
        let decision = plan_packet_promotion(
            command.packet(),
            &snapshots,
            &facts,
            command.actor(),
            promoted_at,
        );
        let PacketPromotionDecision::Accepted(plan) = decision else {
            return Err(PromotionError::InconsistentDecision);
        };
        if plan.is_noop() {
            return Ok(PacketPromotionOutcome::Promoted(PacketPromotionResult::from_plan(&plan)));
        }

        let paths = plan.advanced().iter().map(|item| item.path().clone()).collect::<Vec<_>>();
        let sources = self.store.load_batch(&paths)?;
        let source_by_path = sources
            .into_iter()
            .map(|source| (source.path().clone(), source))
            .collect::<BTreeMap<_, _>>();
        let entries = plan
            .advanced()
            .iter()
            .map(|promotion| {
                source_by_path
                    .get(promotion.path())
                    .cloned()
                    .map(|source| BatchPromotionEntry::new(source, promotion.clone()))
                    .ok_or_else(|| PromotionError::TargetNotFound(promotion.path().clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.store.commit_batch(&entries)?;
        Ok(PacketPromotionOutcome::Promoted(PacketPromotionResult::from_plan(&plan)))
    }
}

fn packet_facts(
    snapshots: &[ArtifactSnapshot],
    command: &PromotePacketCommand,
) -> PacketPromotionFacts {
    let relationship_diagnostics = validate_relationships(snapshots)
        .into_iter()
        .chain(validate_reciprocal_relationships(snapshots))
        .chain(validate_relationship_cycles(snapshots))
        .collect::<Vec<_>>();
    let mut facts = PacketPromotionFacts::default();
    for snapshot in snapshots {
        let Some(source) = domain::state_of(snapshot) else { continue };
        let Some(target) = next_lifecycle_state(source) else { continue };
        let base = facts_for(snapshot, &relationship_diagnostics);
        let external = command.facts().facts_for(snapshot, target);
        let combined = base.conjoined_with(&external);
        if let Some(id) = snapshot.id() {
            let combined = facts
                .facts_for_id(id.as_str())
                .map_or(combined.clone(), |existing| existing.conjoined_with(&combined));
            facts.insert_id(id.as_str(), combined);
        } else {
            let combined = facts
                .facts_for_kind(snapshot.kind())
                .map_or(combined.clone(), |existing| existing.conjoined_with(&combined));
            facts.insert_kind(snapshot.kind(), combined);
        }
    }
    facts
}

fn facts_for(
    snapshot: &ArtifactSnapshot,
    relationship_diagnostics: &[Diagnostic],
) -> PromotionFacts {
    let validation_diagnostics = validate_artifact(snapshot);
    let mut relationships = relationship_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path() == snapshot.path())
        .cloned()
        .collect::<Vec<_>>();
    relationships.sort_by(|left, right| {
        left.rule_id()
            .cmp(right.rule_id())
            .then_with(|| left.path().cmp(right.path()))
            .then_with(|| left.message().cmp(right.message()))
    });
    let blockers = match snapshot.metadata().get("blockers") {
        Some(domain::MetadataValue::Sequence(values)) => values.clone(),
        Some(domain::MetadataValue::Scalar(value)) if !value.is_empty() => vec![value.clone()],
        Some(
            domain::MetadataValue::Null
            | domain::MetadataValue::Mapping
            | domain::MetadataValue::Scalar(_),
        )
        | None => Vec::new(),
    };
    let archive_location = snapshot.path().as_str().starts_with("specs/archive/");
    let checks_pass = validation_diagnostics.is_empty();
    PromotionFacts::passing()
        .with_review_entry_checks(checks_pass)
        .with_approval_checks(checks_pass)
        .with_archive_location(archive_location)
        .with_blockers(blockers)
        .with_validation_diagnostics(validation_diagnostics)
        .with_relationship_diagnostics(relationships)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use domain::{
        ArtifactKind, ArtifactPath, ArtifactSnapshot, DocumentSnapshot, FeatureSnapshot,
        ImplementationPacketRef, Metadata, PacketPromotionFacts, PromotionActor, PromotionFacts,
        PromotionTimestamp,
    };

    use super::*;
    use crate::fake::FixedClock;
    use crate::{ArtifactCandidate, ArtifactIdentitySource, SourceArtifact, ValidationError};

    #[derive(Clone, Debug)]
    struct CountingSource {
        candidates: Vec<ArtifactCandidate>,
        calls: Rc<Cell<usize>>,
    }

    impl ArtifactIdentitySource for CountingSource {
        fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.candidates.clone())
        }
    }

    #[derive(Debug)]
    struct TestPacketStore {
        sources: BTreeMap<ArtifactPath, SourceArtifact>,
        commit_calls: usize,
    }

    impl ArtifactPacketPromotionStore for TestPacketStore {
        fn load_batch(
            &self,
            paths: &[ArtifactPath],
        ) -> Result<Vec<SourceArtifact>, PromotionError> {
            paths
                .iter()
                .map(|path| {
                    self.sources
                        .get(path)
                        .cloned()
                        .ok_or_else(|| PromotionError::TargetNotFound(path.clone()))
                })
                .collect()
        }

        fn commit_batch(
            &mut self,
            entries: &[BatchPromotionEntry],
        ) -> Result<Vec<SourceArtifact>, PromotionError> {
            self.commit_calls += 1;
            Ok(entries.iter().map(|entry| entry.source().clone()).collect())
        }
    }

    fn path(value: &str) -> ArtifactPath {
        match ArtifactPath::try_new(value) {
            Ok(path) => path,
            Err(_) => std::process::abort(),
        }
    }

    fn actor() -> PromotionActor {
        match PromotionActor::try_new("reviewer-1") {
            Ok(actor) => actor,
            Err(_) => std::process::abort(),
        }
    }

    fn snapshot(path_value: &str, status: &str) -> ArtifactSnapshot {
        ArtifactSnapshot::new(
            path(path_value),
            ArtifactKind::Gherkin,
            Metadata::new(),
            DocumentSnapshot::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Some(FeatureSnapshot::new(
                    Some("US-007".to_string()),
                    Some(status.to_string()),
                    Some("Feature".to_string()),
                    1,
                    true,
                    true,
                    true,
                )),
            ),
        )
    }

    fn candidate(snapshot: &ArtifactSnapshot) -> ArtifactCandidate {
        ArtifactCandidate::from_snapshot(snapshot.clone())
    }

    fn packet() -> ImplementationPacketRef {
        match ImplementationPacketRef::try_new("specs/007-packet") {
            Ok(packet) => packet,
            Err(_) => std::process::abort(),
        }
    }

    #[test]
    fn discovers_once_and_commits_an_accepted_packet_once() {
        let snapshots = [
            snapshot("specs/007-packet/user-story.md", "draft"),
            snapshot("specs/007-packet/scenarios.feature", "in-review"),
        ];
        let calls = Rc::new(Cell::new(0));
        let source = CountingSource {
            candidates: snapshots.iter().map(candidate).collect(),
            calls: calls.clone(),
        };
        let store = TestPacketStore {
            sources: snapshots
                .iter()
                .map(|item| {
                    (item.path().clone(), SourceArtifact::new(item.path().clone(), "source"))
                })
                .collect(),
            commit_calls: 0,
        };
        let facts = PacketPromotionFacts::default().with_kind(
            ArtifactKind::Gherkin,
            PromotionFacts::passing().with_human_approval_confirmed(true),
        );
        let command = PromotePacketCommand::new(packet(), actor()).with_facts(facts);
        let mut promoter = PacketPromoter::new(
            source,
            store,
            FixedClock::new(PromotionTimestamp::from_unix_seconds(9)),
        );

        let result = promoter.promote(&command);

        assert!(matches!(result, Ok(PacketPromotionOutcome::Promoted(_))));
        assert_eq!(calls.get(), 1);
        assert_eq!(promoter.store.commit_calls, 1);
    }

    #[test]
    fn missing_confirmation_rejects_without_batch_commit() {
        let snapshot = snapshot("specs/007-packet/scenarios.feature", "in-review");
        let source =
            CountingSource { candidates: vec![candidate(&snapshot)], calls: Rc::new(Cell::new(0)) };
        let store = TestPacketStore {
            sources: BTreeMap::from([(
                snapshot.path().clone(),
                SourceArtifact::new(snapshot.path().clone(), "source"),
            )]),
            commit_calls: 0,
        };
        let command = PromotePacketCommand::new(packet(), actor());
        let mut promoter =
            PacketPromoter::new(source, store, FixedClock::failing("clock should not be used"));

        let result = promoter.promote(&command);

        assert!(matches!(result, Ok(PacketPromotionOutcome::Rejected(_))));
        assert_eq!(promoter.store.commit_calls, 0);
    }

    #[test]
    fn all_skipped_packet_is_successful_noop_without_clock_or_commit() {
        let snapshot = snapshot("specs/007-packet/scenarios.feature", "archived");
        let source =
            CountingSource { candidates: vec![candidate(&snapshot)], calls: Rc::new(Cell::new(0)) };
        let store = TestPacketStore { sources: BTreeMap::new(), commit_calls: 0 };
        let command = PromotePacketCommand::new(packet(), actor());
        let clock = FixedClock::failing("clock should not be used");
        let mut promoter = PacketPromoter::new(source, store, clock);

        let result = promoter.promote(&command);

        assert!(
            matches!(result, Ok(PacketPromotionOutcome::Promoted(result)) if result.advanced().is_empty() && result.skipped().len() == 1)
        );
        assert_eq!(promoter.store.commit_calls, 0);
        assert_eq!(promoter.clock.calls(), 0);
    }
}
