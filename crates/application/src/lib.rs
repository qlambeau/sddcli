//! Orchestrates artifact validation through a narrow source port.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod error;
mod ports;
mod validate;

pub use error::ValidationError;
pub use ports::{ArtifactCandidate, ArtifactIdentitySource, ArtifactSource};
pub use validate::{
    CycleValidator, IdentityValidator, ReciprocalRelationshipValidator, RelationshipValidator,
    Validator,
};

#[cfg(any(test, feature = "test-doubles"))]
pub use validate::fake;
