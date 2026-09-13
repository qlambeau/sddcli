use domain::{
    ArtifactPath, Diagnostic, LifecycleState, MetadataValue, PromotionActor, PromotionDecision,
    PromotionFacts, PromotionPlan, PromotionRequest, PromotionTimestamp, completion_facts,
    decide_promotion, validate_artifact, validate_reciprocal_relationships,
    validate_relationship_cycles, validate_relationships,
};

use crate::{ArtifactIdentitySource, ArtifactPromotionStore, Clock, PromotionError};

/// Requests promotion of one recognized artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "REQ-006 command carries independent external workflow confirmations"
)]
pub struct PromoteArtifactCommand {
    path: ArtifactPath,
    target: LifecycleState,
    actor: PromotionActor,
    human_approval_confirmed: bool,
    implementation_evidence_recorded: bool,
    closed_release_conditions_satisfied: bool,
    approved_successor_with_valid_links: bool,
}

impl PromoteArtifactCommand {
    /// Creates a promotion command with external workflow facts defaulting to present.
    #[must_use]
    pub const fn new(path: ArtifactPath, target: LifecycleState, actor: PromotionActor) -> Self {
        Self {
            path,
            target,
            actor,
            human_approval_confirmed: true,
            implementation_evidence_recorded: true,
            closed_release_conditions_satisfied: true,
            approved_successor_with_valid_links: true,
        }
    }

    /// Records whether external human approval was confirmed.
    #[must_use]
    pub const fn with_human_approval_confirmed(mut self, confirmed: bool) -> Self {
        self.human_approval_confirmed = confirmed;
        self
    }

    /// Records whether implementation and verification evidence is present.
    #[must_use]
    pub const fn with_implementation_evidence_recorded(mut self, recorded: bool) -> Self {
        self.implementation_evidence_recorded = recorded;
        self
    }

    /// Records whether release closure conditions are satisfied.
    #[must_use]
    pub const fn with_closed_release_conditions_satisfied(mut self, satisfied: bool) -> Self {
        self.closed_release_conditions_satisfied = satisfied;
        self
    }

    /// Records whether supersession prerequisites are satisfied.
    #[must_use]
    pub const fn with_approved_successor_with_valid_links(mut self, valid: bool) -> Self {
        self.approved_successor_with_valid_links = valid;
        self
    }

    /// Returns the target artifact path.
    #[must_use]
    pub const fn path(&self) -> &ArtifactPath {
        &self.path
    }
}

/// Reports the observable result of a promotion request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromotionOutcome {
    /// A real transition was committed.
    Promoted(PromotionPlan),
    /// The active artifact was already in the requested state and no write occurred.
    Idempotent,
    /// The request failed before mutation with complete diagnostics.
    Rejected(Vec<Diagnostic>),
}

/// Orchestrates one-artifact lifecycle promotion through application-owned ports.
pub struct Promoter<S, P, C> {
    source: S,
    store: P,
    clock: C,
}

impl<S, P, C> Promoter<S, P, C>
where
    S: ArtifactIdentitySource,
    P: ArtifactPromotionStore,
    C: Clock,
{
    /// Creates a promoter from its repository source, promotion store, and clock.
    #[must_use]
    pub const fn new(source: S, store: P, clock: C) -> Self {
        Self { source, store, clock }
    }

    /// Promotes one artifact when every applicable prerequisite passes.
    ///
    /// # Errors
    ///
    /// Returns typed operational errors for discovery, source loading, clock, conflict, or commit
    /// failures. Artifact prerequisite failures are returned as [`PromotionOutcome::Rejected`].
    pub fn promote(
        &mut self,
        command: &PromoteArtifactCommand,
    ) -> Result<PromotionOutcome, PromotionError> {
        let source_artifact = self.store.load(command.path())?;
        let candidates = self.source.discover_identities()?;
        let snapshots = candidates
            .iter()
            .filter_map(|candidate| candidate.snapshot().cloned())
            .collect::<Vec<_>>();
        let Some(snapshot) = snapshots.iter().find(|snapshot| snapshot.path() == command.path())
        else {
            return Err(PromotionError::TargetNotFound(command.path().clone()));
        };

        let facts = facts_for(snapshot, &snapshots, command);
        let request = PromotionRequest::new(command.target, command.actor.clone());
        let preliminary =
            decide_promotion(snapshot, &facts, &request, PromotionTimestamp::from_unix_seconds(0));

        match preliminary {
            PromotionDecision::Rejected(diagnostics) => Ok(PromotionOutcome::Rejected(diagnostics)),
            PromotionDecision::Idempotent => Ok(PromotionOutcome::Idempotent),
            PromotionDecision::Accepted(_) => {
                let promoted_at = self.clock.now()?;
                let decision = decide_promotion(snapshot, &facts, &request, promoted_at);
                let PromotionDecision::Accepted(plan) = decision else {
                    return Err(PromotionError::InconsistentDecision);
                };
                self.store.commit(&source_artifact, &plan)?;
                Ok(PromotionOutcome::Promoted(plan))
            }
        }
    }
}

fn facts_for(
    snapshot: &domain::ArtifactSnapshot,
    snapshots: &[domain::ArtifactSnapshot],
    command: &PromoteArtifactCommand,
) -> PromotionFacts {
    let validation_diagnostics = validate_artifact(snapshot);
    let mut relationship_diagnostics = validate_relationships(snapshots)
        .into_iter()
        .chain(validate_reciprocal_relationships(snapshots))
        .chain(validate_relationship_cycles(snapshots))
        .filter(|diagnostic| diagnostic.path() == snapshot.path())
        .collect::<Vec<_>>();
    relationship_diagnostics.sort_by(|left, right| left.rule_id().cmp(right.rule_id()));
    let blockers = blockers(snapshot);
    let archive_location = snapshot.path().as_str().starts_with("specs/archive/");
    let checks_pass = validation_diagnostics.is_empty();
    let completion = completion_facts(snapshot, snapshots, command.target);

    PromotionFacts::passing()
        .with_review_entry_checks(checks_pass)
        .with_approval_checks(checks_pass)
        .with_human_approval_confirmed(command.human_approval_confirmed)
        .with_implementation_evidence(command.implementation_evidence_recorded)
        .with_archive_location(archive_location)
        .with_closed_release_conditions(command.closed_release_conditions_satisfied)
        .with_supersession(command.approved_successor_with_valid_links)
        .with_blockers(blockers)
        .with_validation_diagnostics(validation_diagnostics)
        .with_relationship_diagnostics(relationship_diagnostics)
        .conjoined_with(&completion)
}

fn blockers(snapshot: &domain::ArtifactSnapshot) -> Vec<String> {
    match snapshot.metadata().get("blockers") {
        Some(MetadataValue::Sequence(values)) => values.clone(),
        Some(MetadataValue::Scalar(value)) if !value.is_empty() => vec![value.clone()],
        Some(MetadataValue::Null | MetadataValue::Mapping | MetadataValue::Scalar(_)) | None => {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use domain::{ArtifactKind, ArtifactSnapshot, DocumentSnapshot, Metadata};

    use super::*;
    use crate::{
        ArtifactCandidate, SourceArtifact,
        fake::{FixedClock, InMemoryArtifactIdentitySource, InMemoryPromotionStore},
    };

    fn path() -> ArtifactPath {
        match ArtifactPath::try_new("specs/feature/user-story.md") {
            Ok(path) => path,
            Err(_) => std::process::abort(),
        }
    }

    fn snapshot(status: &str) -> ArtifactSnapshot {
        let mut metadata = Metadata::new();
        metadata.insert_scalar("status", status);
        metadata.insert_scalar("id", "US-001");
        metadata.insert_scalar("type", "user-story");
        metadata.insert_scalar("title", "Title");
        metadata.insert_scalar("created", "2026-09-07");
        metadata.insert_scalar("updated", "2026-09-07");
        metadata.insert_scalar("owner", "owner");
        metadata.insert_scalar("parent", "PRD-001");
        metadata.insert_scalar("epic", "EPIC-001");
        metadata.insert_scalar("feature", "feature");
        metadata.insert_sequence("depends_on", Vec::<String>::new());
        metadata.insert_sequence("requires", Vec::<String>::new());
        metadata.insert_sequence("blockers", Vec::<String>::new());
        metadata.insert_sequence("related", Vec::<String>::new());
        ArtifactSnapshot::new(path(), ArtifactKind::UserStory, metadata, DocumentSnapshot::empty())
    }

    fn actor() -> PromotionActor {
        match PromotionActor::try_new("agent-1") {
            Ok(actor) => actor,
            Err(_) => std::process::abort(),
        }
    }

    /// Covers: REQ-006 FR-004 and FR-012 — idempotent requests do not call clock or commit.
    #[test]
    fn idempotent_request_does_not_write_or_read_clock() {
        let source = InMemoryArtifactIdentitySource::new(vec![ArtifactCandidate::from_snapshot(
            snapshot("approved"),
        )]);
        let store = InMemoryPromotionStore::new(SourceArtifact::new(path(), "original"));
        let clock = FixedClock::failing("clock should not be used");
        let mut promoter = Promoter::new(source, store, clock);

        let command = PromoteArtifactCommand::new(path(), LifecycleState::Approved, actor());
        let result = promoter.promote(&command);

        assert_eq!(result, Ok(PromotionOutcome::Idempotent));
        assert_eq!(promoter.store.commit_count(), 0);
    }

    /// Covers: REQ-006 FR-007 and FR-013 — rejected requests aggregate diagnostics and do not write.
    #[test]
    fn rejected_request_does_not_commit() {
        let source = InMemoryArtifactIdentitySource::new(vec![ArtifactCandidate::from_snapshot(
            snapshot("approved"),
        )]);
        let store = InMemoryPromotionStore::new(SourceArtifact::new(path(), "original"));
        let clock = FixedClock::new(PromotionTimestamp::from_unix_seconds(5));
        let mut promoter = Promoter::new(source, store, clock);
        let command = PromoteArtifactCommand::new(path(), LifecycleState::Implemented, actor())
            .with_implementation_evidence_recorded(false);

        let result = promoter.promote(&command);

        let Ok(PromotionOutcome::Rejected(diagnostics)) = result else { std::process::abort() };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("EVIDENCE_MISSING"))
        );
        assert_eq!(promoter.store.commit_count(), 0);
    }
}
