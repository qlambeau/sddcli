use thiserror::Error;

/// Describes an operational failure that prevents artifact discovery.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ValidationError {
    /// Indicates that the source could not establish the repository artifact set.
    #[error("artifact discovery failed: {0}")]
    Discovery(String),
}

/// Describes an operational failure during artifact promotion.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum PromotionError {
    /// Indicates that candidate discovery failed.
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// Indicates that the target artifact was not found in discovered candidates.
    #[error("promotion target was not found: {0}")]
    TargetNotFound(domain::ArtifactPath),
    /// Indicates that the target source could not be loaded.
    #[error("promotion source could not be loaded: {0}")]
    Source(String),
    /// Indicates that the source changed between preflight and commit.
    #[error("promotion source changed before commit: {0}")]
    Conflict(domain::ArtifactPath),
    /// Indicates that the target format cannot be patched.
    #[error("promotion target format is unsupported: {0}")]
    UnsupportedFormat(domain::ArtifactPath),
    /// Indicates that the artifact could not be written.
    #[error("promotion write failed: {0}")]
    Write(String),
    /// Indicates that the clock could not supply a timestamp.
    #[error("promotion clock failed: {0}")]
    Clock(String),
    /// Indicates that a repeated pure decision changed unexpectedly.
    #[error("promotion decision changed between preflight and commit")]
    InconsistentDecision,
}
