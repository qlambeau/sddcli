//! Defines pure SDD artifact validation models, rules, and deterministic reports.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod artifact;
mod diagnostic;
mod document;
mod identity;
mod report;
mod rules;

pub use artifact::{
    ArtifactId, ArtifactKind, ArtifactPath, ArtifactSnapshot, Metadata, MetadataValue,
};
pub use diagnostic::{Diagnostic, Location, RuleId, Severity};
pub use document::{ChecklistItem, DocumentSnapshot, FeatureSnapshot, Heading, SourceLine};
pub use identity::validate_identities;
pub use report::{ArtifactResult, ArtifactStatus, OverallStatus, ValidationReport};
pub use rules::validate_artifact;
