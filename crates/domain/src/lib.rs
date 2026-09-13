//! Defines pure SDD artifact validation models, rules, and deterministic reports.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod artifact;
mod cycles;
mod diagnostic;
mod document;
mod identity;
mod packet;
mod promotion;
mod readiness;
mod reciprocal;
mod relationships;
mod report;
mod rules;
mod schema;

pub use artifact::{
    ArtifactId, ArtifactKind, ArtifactPath, ArtifactSnapshot, Metadata, MetadataValue,
};
pub use cycles::validate_relationship_cycles;
pub use diagnostic::{Diagnostic, Location, RuleId, Severity};
pub use document::{
    ChecklistItem, DocumentSnapshot, FeatureSnapshot, Heading, ScenarioCoverage, SourceLine,
};
pub use identity::validate_identities;
pub use packet::{
    ImplementationPacketRef, PacketParticipant, PacketParticipantRole, PacketPromotionDecision,
    PacketPromotionFacts, PacketPromotionPlan, PacketPromotionStep, PacketReferenceError,
    next_lifecycle_state, plan_packet_promotion,
};
pub use promotion::{
    LifecycleState, PromotionActor, PromotionActorError, PromotionDecision, PromotionFacts,
    PromotionPlan, PromotionRequest, PromotionTimestamp, decide_promotion, state_of,
};
pub use readiness::{SpecReadyDecision, SpecReadyResult, evaluate_spec_ready};
pub use reciprocal::validate_reciprocal_relationships;
pub use relationships::validate_relationships;
pub use report::{ArtifactResult, ArtifactStatus, OverallStatus, ValidationReport};
pub use rules::validate_artifact;
pub use schema::validate_schema_links;
