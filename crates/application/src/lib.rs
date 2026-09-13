//! Orchestrates artifact validation through a narrow source port.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod error;
mod ports;
mod promotion;
mod validate;

pub use error::{PromotionError, ValidationError};
pub use ports::{
    ArtifactCandidate, ArtifactIdentitySource, ArtifactPromotionStore, ArtifactSource, Clock,
    SourceArtifact,
};
pub use promotion::{PromoteArtifactCommand, Promoter, PromotionOutcome};
pub use validate::{
    CycleValidator, IdentityValidator, ReciprocalRelationshipValidator, RelationshipValidator,
    Validator,
};

#[cfg(any(test, feature = "test-doubles"))]
pub mod fake {
    //! Test doubles for application ports.

    pub use crate::ports::fake::{FixedClock, InMemoryPromotionStore};
    pub use crate::validate::fake::{InMemoryArtifactIdentitySource, InMemoryArtifactSource};
}
