use thiserror::Error;

/// Describes an operational failure that prevents artifact discovery.
#[derive(Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ValidationError {
    /// Indicates that the source could not establish the repository artifact set.
    #[error("artifact discovery failed: {0}")]
    Discovery(String),
}
