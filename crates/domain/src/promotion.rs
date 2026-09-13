use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{ArtifactPath, ArtifactSnapshot, Diagnostic, MetadataValue, Severity};

/// Represents a supported lifecycle state for artifact promotion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LifecycleState {
    /// The artifact is being authored.
    Draft,
    /// The artifact is awaiting semantic review.
    InReview,
    /// The artifact has been semantically approved.
    Approved,
    /// The artifact has been implemented and verified.
    Implemented,
    /// The artifact has been archived after release.
    Archived,
    /// The artifact has been replaced by an approved successor.
    Superseded,
}

impl LifecycleState {
    /// Parses a lifecycle state accepted by the promotion policy.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "in-review" => Some(Self::InReview),
            "approved" => Some(Self::Approved),
            "implemented" => Some(Self::Implemented),
            "archived" => Some(Self::Archived),
            "superseded" => Some(Self::Superseded),
            _ => None,
        }
    }

    /// Returns the canonical serialized token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in-review",
            Self::Approved => "approved",
            Self::Implemented => "implemented",
            Self::Archived => "archived",
            Self::Superseded => "superseded",
        }
    }

    /// Returns whether the state is active for same-state no-op handling.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Draft | Self::InReview | Self::Approved | Self::Implemented)
    }
}

/// Identifies the actor requesting a promotion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionActor(String);

impl PromotionActor {
    /// Creates a non-empty actor identity.
    ///
    /// # Errors
    ///
    /// Returns [`PromotionActorError::Empty`] when the identity is blank.
    pub fn try_new(value: impl Into<String>) -> Result<Self, PromotionActorError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(PromotionActorError::Empty);
        }
        Ok(Self(value))
    }

    /// Returns the supplied actor identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explains why a promotion actor identity was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionActorError {
    /// The actor identity was empty or whitespace.
    Empty,
}

impl Display for PromotionActorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("promotion actor is empty")
    }
}

impl Error for PromotionActorError {}

/// UTC Unix seconds used in promotion metadata.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PromotionTimestamp(i64);

impl PromotionTimestamp {
    /// Creates a timestamp from UTC Unix seconds.
    #[must_use]
    pub const fn from_unix_seconds(value: i64) -> Self {
        Self(value)
    }

    /// Returns UTC Unix seconds.
    #[must_use]
    pub const fn as_unix_seconds(self) -> i64 {
        self.0
    }
}

/// Contains caller and target data for a promotion decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionRequest {
    target: LifecycleState,
    actor: PromotionActor,
}

impl PromotionRequest {
    /// Creates a promotion request.
    #[must_use]
    pub const fn new(target: LifecycleState, actor: PromotionActor) -> Self {
        Self { target, actor }
    }

    /// Returns the requested target state.
    #[must_use]
    pub const fn target(&self) -> LifecycleState {
        self.target
    }

    /// Returns the supplied actor identity.
    #[must_use]
    pub const fn actor(&self) -> &PromotionActor {
        &self.actor
    }
}

/// Captures normalized prerequisite facts used by the pure promotion policy.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "REQ-006 facts intentionally model independent prerequisite pass/fail outcomes"
)]
pub struct PromotionFacts {
    review_entry_checks_pass: bool,
    approval_checks_pass: bool,
    human_approval_confirmed: bool,
    implementation_evidence_recorded: bool,
    archive_location_canonical: bool,
    closed_release_conditions_satisfied: bool,
    approved_successor_with_valid_links: bool,
    blockers: Vec<String>,
    relationship_diagnostics: Vec<Diagnostic>,
    validation_diagnostics: Vec<Diagnostic>,
}

impl Default for PromotionFacts {
    fn default() -> Self {
        Self {
            review_entry_checks_pass: true,
            approval_checks_pass: true,
            human_approval_confirmed: true,
            implementation_evidence_recorded: true,
            archive_location_canonical: true,
            closed_release_conditions_satisfied: true,
            approved_successor_with_valid_links: true,
            blockers: Vec::new(),
            relationship_diagnostics: Vec::new(),
            validation_diagnostics: Vec::new(),
        }
    }
}

impl PromotionFacts {
    /// Creates facts whose prerequisite checks all pass.
    #[must_use]
    pub fn passing() -> Self {
        Self::default()
    }

    /// Sets whether review-entry checks pass.
    #[must_use]
    pub const fn with_review_entry_checks(mut self, pass: bool) -> Self {
        self.review_entry_checks_pass = pass;
        self
    }

    /// Sets whether approval checks pass.
    #[must_use]
    pub const fn with_approval_checks(mut self, pass: bool) -> Self {
        self.approval_checks_pass = pass;
        self
    }

    /// Sets whether external human approval was confirmed.
    #[must_use]
    pub const fn with_human_approval_confirmed(mut self, confirmed: bool) -> Self {
        self.human_approval_confirmed = confirmed;
        self
    }

    /// Sets whether implementation and verification evidence is recorded.
    #[must_use]
    pub const fn with_implementation_evidence(mut self, recorded: bool) -> Self {
        self.implementation_evidence_recorded = recorded;
        self
    }

    /// Sets whether the artifact is already in the canonical archive location.
    #[must_use]
    pub const fn with_archive_location(mut self, canonical: bool) -> Self {
        self.archive_location_canonical = canonical;
        self
    }

    /// Sets whether closed release conditions are satisfied.
    #[must_use]
    pub const fn with_closed_release_conditions(mut self, satisfied: bool) -> Self {
        self.closed_release_conditions_satisfied = satisfied;
        self
    }

    /// Sets whether an approved successor and valid links are present.
    #[must_use]
    pub const fn with_supersession(mut self, valid: bool) -> Self {
        self.approved_successor_with_valid_links = valid;
        self
    }

    /// Adds unresolved blockers.
    #[must_use]
    pub fn with_blockers(mut self, blockers: Vec<String>) -> Self {
        self.blockers = blockers;
        self
    }

    /// Adds relationship diagnostics that must block mutation.
    #[must_use]
    pub fn with_relationship_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.relationship_diagnostics = diagnostics;
        self
    }

    /// Adds validation diagnostics that must block mutation.
    #[must_use]
    pub fn with_validation_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.validation_diagnostics = diagnostics;
        self
    }
}

/// Describes one real promotion to persist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionPlan {
    path: ArtifactPath,
    source: LifecycleState,
    target: LifecycleState,
    actor: PromotionActor,
    promoted_at: PromotionTimestamp,
}

impl PromotionPlan {
    /// Creates a promotion plan.
    #[must_use]
    pub const fn new(
        path: ArtifactPath,
        source: LifecycleState,
        target: LifecycleState,
        actor: PromotionActor,
        promoted_at: PromotionTimestamp,
    ) -> Self {
        Self { path, source, target, actor, promoted_at }
    }

    /// Returns the promoted artifact path.
    #[must_use]
    pub const fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the source lifecycle state.
    #[must_use]
    pub const fn source(&self) -> LifecycleState {
        self.source
    }

    /// Returns the target lifecycle state.
    #[must_use]
    pub const fn target(&self) -> LifecycleState {
        self.target
    }

    /// Returns the actor identity.
    #[must_use]
    pub const fn actor(&self) -> &PromotionActor {
        &self.actor
    }

    /// Returns the promotion timestamp.
    #[must_use]
    pub const fn promoted_at(&self) -> PromotionTimestamp {
        self.promoted_at
    }
}

/// Represents the pure promotion decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromotionDecision {
    /// A real state change is accepted.
    Accepted(PromotionPlan),
    /// The active artifact already has the requested state and must not be written.
    Idempotent,
    /// The request is rejected with ordered diagnostics.
    Rejected(Vec<Diagnostic>),
}

/// Decides whether one artifact can be promoted without performing I/O.
#[must_use]
pub fn decide_promotion(
    snapshot: &ArtifactSnapshot,
    facts: &PromotionFacts,
    request: &PromotionRequest,
    promoted_at: PromotionTimestamp,
) -> PromotionDecision {
    let Some(source) = state_of(snapshot) else {
        return PromotionDecision::Rejected(vec![diagnostic(
            snapshot.path(),
            "STATE_MISSING",
            "artifact lifecycle state is missing or unsupported",
            "set the artifact status to a supported lifecycle value",
        )]);
    };

    if source == request.target() && source.is_active() {
        return PromotionDecision::Idempotent;
    }

    let mut diagnostics = Vec::new();
    if matches!(source, LifecycleState::Archived | LifecycleState::Superseded) {
        diagnostics.push(diagnostic(
            snapshot.path(),
            "TERMINAL_STATE",
            format!("artifact in `{}` cannot be promoted", source.as_str()),
            "do not reopen archived or superseded artifacts through promotion",
        ));
    }
    if !is_allowed_transition(source, request.target()) {
        diagnostics.push(diagnostic(
            snapshot.path(),
            "INVALID_TRANSITION",
            format!("cannot promote from `{}` to `{}`", source.as_str(), request.target().as_str()),
            "request the next permitted lifecycle state",
        ));
    }

    diagnostics.extend(facts.validation_diagnostics.clone());
    diagnostics.extend(facts.relationship_diagnostics.clone());
    for blocker in &facts.blockers {
        diagnostics.push(diagnostic(
            snapshot.path(),
            "BLOCKER_UNRESOLVED",
            format!("artifact has unresolved blocker `{blocker}`"),
            "resolve blockers before promotion",
        ));
    }
    add_target_guards(snapshot.path(), request.target(), facts, &mut diagnostics);

    if !diagnostics.is_empty() {
        diagnostics.sort_by(|left, right| {
            left.rule_id()
                .cmp(right.rule_id())
                .then_with(|| left.location().cmp(&right.location()))
                .then_with(|| left.message().cmp(right.message()))
        });
        return PromotionDecision::Rejected(diagnostics);
    }

    PromotionDecision::Accepted(PromotionPlan::new(
        snapshot.path().clone(),
        source,
        request.target(),
        request.actor().clone(),
        promoted_at,
    ))
}

/// Returns the normalized lifecycle state of an artifact snapshot.
#[must_use]
pub fn state_of(snapshot: &ArtifactSnapshot) -> Option<LifecycleState> {
    if let Some(feature) = snapshot.document().feature() {
        return feature.status().and_then(LifecycleState::parse);
    }
    match snapshot.metadata().get("status") {
        Some(MetadataValue::Scalar(value)) => LifecycleState::parse(value),
        _ => None,
    }
}

fn is_allowed_transition(source: LifecycleState, target: LifecycleState) -> bool {
    matches!(
        (source, target),
        (LifecycleState::Draft, LifecycleState::InReview)
            | (LifecycleState::InReview, LifecycleState::Approved)
            | (LifecycleState::Approved, LifecycleState::Implemented | LifecycleState::Superseded)
            | (LifecycleState::Implemented, LifecycleState::Archived)
    )
}

fn add_target_guards(
    path: &ArtifactPath,
    target: LifecycleState,
    facts: &PromotionFacts,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match target {
        LifecycleState::InReview if !facts.review_entry_checks_pass => {
            diagnostics.push(diagnostic(
                path,
                "REVIEW_ENTRY",
                "review-entry checks failed",
                "satisfy review-entry checks before promotion",
            ));
        }
        LifecycleState::Approved => {
            if !facts.approval_checks_pass {
                diagnostics.push(diagnostic(
                    path,
                    "APPROVAL_CHECKS",
                    "approval checks failed",
                    "satisfy approval checks before approval promotion",
                ));
            }
            if !facts.human_approval_confirmed {
                diagnostics.push(diagnostic(
                    path,
                    "HUMAN_APPROVAL",
                    "external human approval confirmation is missing",
                    "confirm human approval in the surrounding workflow",
                ));
            }
        }
        LifecycleState::Implemented if !facts.implementation_evidence_recorded => {
            diagnostics.push(diagnostic(
                path,
                "EVIDENCE_MISSING",
                "implementation or verification evidence is missing",
                "record implementation and verification evidence before promotion",
            ));
        }
        LifecycleState::Archived => {
            if !facts.archive_location_canonical {
                diagnostics.push(diagnostic(
                    path,
                    "ARCHIVE_LOCATION",
                    "artifact is not in its canonical archive location",
                    "move the artifact through the release workflow before archival promotion",
                ));
            }
            if !facts.closed_release_conditions_satisfied {
                diagnostics.push(diagnostic(
                    path,
                    "RELEASE_CONDITIONS",
                    "closed release conditions are not satisfied",
                    "satisfy release closure conditions before archival promotion",
                ));
            }
        }
        LifecycleState::Superseded if !facts.approved_successor_with_valid_links => {
            diagnostics.push(diagnostic(
                path,
                "SUPERSESSION_PREREQUISITE",
                "approved successor with valid supersession links is missing",
                "approve a successor and set reciprocal supersession links",
            ));
        }
        LifecycleState::Draft
        | LifecycleState::InReview
        | LifecycleState::Implemented
        | LifecycleState::Superseded => {}
    }
}

fn diagnostic(
    path: &ArtifactPath,
    rule: &str,
    message: impl Into<String>,
    remediation: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        path.clone(),
        None,
        format!("ARTIFACT.PROMOTION.{rule}"),
        Severity::Error,
        message,
        remediation,
    )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;
    use crate::{ArtifactKind, DocumentSnapshot, Metadata};

    fn actor() -> PromotionActor {
        match PromotionActor::try_new("agent-1") {
            Ok(actor) => actor,
            Err(_) => std::process::abort(),
        }
    }

    fn snapshot(status: &str) -> ArtifactSnapshot {
        let Ok(path) = ArtifactPath::try_new("specs/006-feature/user-story.md") else {
            std::process::abort()
        };
        let mut metadata = Metadata::new();
        metadata.insert_scalar("status", status);
        ArtifactSnapshot::new(path, ArtifactKind::UserStory, metadata, DocumentSnapshot::empty())
    }

    /// Covers: REQ-006 FR-002 and FR-010 — strict forward transitions create metadata plans.
    #[rstest]
    #[case("draft", LifecycleState::InReview)]
    #[case("in-review", LifecycleState::Approved)]
    #[case("approved", LifecycleState::Implemented)]
    #[case("implemented", LifecycleState::Archived)]
    #[case("approved", LifecycleState::Superseded)]
    fn accepts_permitted_transitions(#[case] source: &str, #[case] target: LifecycleState) {
        let request = PromotionRequest::new(target, actor());

        let decision = decide_promotion(
            &snapshot(source),
            &PromotionFacts::passing(),
            &request,
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PromotionDecision::Accepted(plan) = decision else { std::process::abort() };
        assert_eq!(plan.source().as_str(), source);
        assert_eq!(plan.target(), target);
        assert_eq!(plan.actor().as_str(), "agent-1");
        assert_eq!(plan.promoted_at().as_unix_seconds(), 42);
    }

    /// Covers: REQ-006 FR-004 — active same-state requests are idempotent no-ops.
    #[test]
    fn returns_idempotent_for_active_same_state() {
        let request = PromotionRequest::new(LifecycleState::Approved, actor());

        let decision = decide_promotion(
            &snapshot("approved"),
            &PromotionFacts::passing(),
            &request,
            PromotionTimestamp::from_unix_seconds(42),
        );

        assert_eq!(decision, PromotionDecision::Idempotent);
    }

    /// Covers: REQ-006 FR-003 and FR-011 — terminal and invalid requests are rejected.
    #[rstest]
    #[case("draft", LifecycleState::Approved)]
    #[case("approved", LifecycleState::Draft)]
    #[case("archived", LifecycleState::Approved)]
    #[case("superseded", LifecycleState::Approved)]
    fn rejects_invalid_transitions(#[case] source: &str, #[case] target: LifecycleState) {
        let request = PromotionRequest::new(target, actor());

        let decision = decide_promotion(
            &snapshot(source),
            &PromotionFacts::passing(),
            &request,
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PromotionDecision::Rejected(diagnostics) = decision else { std::process::abort() };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("INVALID_TRANSITION"))
        );
    }

    /// Covers: REQ-006 FR-005 through FR-009 — target-specific guards reject missing prerequisites.
    #[rstest]
    #[case("draft", LifecycleState::InReview, PromotionFacts::passing().with_review_entry_checks(false), "REVIEW_ENTRY")]
    #[case("in-review", LifecycleState::Approved, PromotionFacts::passing().with_approval_checks(false), "APPROVAL_CHECKS")]
    #[case("in-review", LifecycleState::Approved, PromotionFacts::passing().with_human_approval_confirmed(false), "HUMAN_APPROVAL")]
    #[case("implemented", LifecycleState::Archived, PromotionFacts::passing().with_archive_location(false), "ARCHIVE_LOCATION")]
    #[case("implemented", LifecycleState::Archived, PromotionFacts::passing().with_closed_release_conditions(false), "RELEASE_CONDITIONS")]
    #[case("approved", LifecycleState::Superseded, PromotionFacts::passing().with_supersession(false), "SUPERSESSION_PREREQUISITE")]
    fn rejects_target_guard_failures(
        #[case] source: &str,
        #[case] target: LifecycleState,
        #[case] facts: PromotionFacts,
        #[case] rule: &str,
    ) {
        let request = PromotionRequest::new(target, actor());

        let decision = decide_promotion(
            &snapshot(source),
            &facts,
            &request,
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PromotionDecision::Rejected(diagnostics) = decision else { std::process::abort() };
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.rule_id().as_str().ends_with(rule)));
    }

    /// Covers: REQ-006 FR-001 — Gherkin status headers provide lifecycle state.
    #[test]
    fn reads_lifecycle_state_from_gherkin_feature_status() {
        let Ok(path) = ArtifactPath::try_new("specs/feature/scenarios.feature") else {
            std::process::abort()
        };
        let feature = crate::FeatureSnapshot::new(
            Some("US-001".to_string()),
            Some("draft".to_string()),
            Some("Feature".to_string()),
            1,
            true,
            true,
            true,
        );
        let artifact = ArtifactSnapshot::new(
            path,
            ArtifactKind::Gherkin,
            Metadata::new(),
            DocumentSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Some(feature)),
        );

        assert_eq!(state_of(&artifact), Some(LifecycleState::Draft));
    }

    /// Covers: REQ-006 FR-011 — missing or unsupported state is diagnostic rather than mutation.
    #[test]
    fn rejects_artifact_without_supported_lifecycle_state() {
        let request = PromotionRequest::new(LifecycleState::InReview, actor());

        let decision = decide_promotion(
            &snapshot("released"),
            &PromotionFacts::passing(),
            &request,
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PromotionDecision::Rejected(diagnostics) = decision else { std::process::abort() };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id().as_str().ends_with("STATE_MISSING"))
        );
        assert_eq!(LifecycleState::parse("released"), None);
        assert_eq!(PromotionActorError::Empty.to_string(), "promotion actor is empty");
        assert!(PromotionActor::try_new(" ").is_err());
    }

    /// Covers: REQ-006 FR-007, FR-009, and FR-013 — all applicable failures are ordered.
    #[test]
    fn aggregates_prerequisite_diagnostics_in_stable_order() {
        let request = PromotionRequest::new(LifecycleState::Implemented, actor());
        let facts = PromotionFacts::passing()
            .with_implementation_evidence(false)
            .with_blockers(vec!["OBS-001".to_string()]);

        let decision = decide_promotion(
            &snapshot("approved"),
            &facts,
            &request,
            PromotionTimestamp::from_unix_seconds(42),
        );

        let PromotionDecision::Rejected(diagnostics) = decision else { std::process::abort() };
        let rules =
            diagnostics.iter().map(|diagnostic| diagnostic.rule_id().as_str()).collect::<Vec<_>>();
        assert_eq!(
            rules,
            vec!["ARTIFACT.PROMOTION.BLOCKER_UNRESOLVED", "ARTIFACT.PROMOTION.EVIDENCE_MISSING"]
        );
    }

    proptest! {
        /// Covers: REQ-006 FR-013 — repeated decisions are deterministic.
        #[test]
        fn decisions_are_deterministic(target in 0usize..6) {
            let states = [
                LifecycleState::Draft,
                LifecycleState::InReview,
                LifecycleState::Approved,
                LifecycleState::Implemented,
                LifecycleState::Archived,
                LifecycleState::Superseded,
            ];
            let state = states.get(target).copied().unwrap_or(LifecycleState::Superseded);
            let request = PromotionRequest::new(state, actor());
            let artifact = snapshot("approved");
            let facts = PromotionFacts::passing().with_implementation_evidence(false);

            let first = decide_promotion(&artifact, &facts, &request, PromotionTimestamp::from_unix_seconds(42));
            let second = decide_promotion(&artifact, &facts, &request, PromotionTimestamp::from_unix_seconds(42));

            prop_assert_eq!(first, second);
        }
    }
}
