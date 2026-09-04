//! Reads canonical active SDD files and maps parser output into application candidates.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod discovery;
mod parser;

pub use discovery::FilesystemArtifactSource;
pub use parser::parse_artifact;
