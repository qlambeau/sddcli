//! Orchestrates artifact validation through a narrow source port.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod error;
mod packet;
mod ports;
mod promotion;
mod readiness;
mod validate;

pub use error::{PromotionError, ReadinessError, ValidationError};
pub use packet::{
    PacketPromoter, PacketPromotionOutcome, PacketPromotionResult, PromotePacketCommand,
};
pub use ports::{
    ArtifactCandidate, ArtifactIdentitySource, ArtifactPacketPromotionStore,
    ArtifactPromotionStore, ArtifactSource, BatchPromotionEntry, Clock, SourceArtifact,
};
pub use promotion::{PromoteArtifactCommand, Promoter, PromotionOutcome};
pub use readiness::{EvaluateSpecReadyCommand, SpecReadyEvaluator};
pub use validate::{
    CycleValidator, IdentityValidator, ReciprocalRelationshipValidator, RelationshipValidator,
    Validator,
};

#[cfg(any(test, feature = "test-doubles"))]
pub mod fake {
    //! Test doubles for application ports.

    pub use crate::ports::fake::{
        FixedClock, InMemoryPacketPromotionStore, InMemoryPromotionStore,
    };
    pub use crate::validate::fake::{InMemoryArtifactIdentitySource, InMemoryArtifactSource};
}
