use domain::{ArtifactPath, ArtifactSnapshot};

use crate::ValidationError;

/// Supplies every candidate at the canonical active artifact locations.
pub trait ArtifactSource {
    /// Returns candidates in any order; the validation report applies stable ordering.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::Discovery`] when the repository-level candidate set cannot
    /// be established. Per-file failures must be represented as candidate diagnostics.
    fn discover(&self) -> Result<Vec<ArtifactCandidate>, ValidationError>;
}

/// Supplies every candidate at the canonical active and historical identity locations.
pub trait ArtifactIdentitySource {
    /// Returns candidates in any order; the identity report applies stable ordering.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::Discovery`] when the repository-wide candidate set cannot
    /// be established. Per-file failures must be represented as candidate diagnostics.
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError>;
}

/// Represents one discovered artifact or one per-file source failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCandidate {
    path: ArtifactPath,
    snapshot: Option<ArtifactSnapshot>,
    diagnostic: Option<domain::Diagnostic>,
}

impl ArtifactCandidate {
    /// Creates a candidate containing a normalized artifact snapshot.
    #[must_use]
    pub fn from_snapshot(snapshot: ArtifactSnapshot) -> Self {
        Self { path: snapshot.path().clone(), snapshot: Some(snapshot), diagnostic: None }
    }

    /// Creates a candidate containing a source-located per-file diagnostic.
    #[must_use]
    pub fn from_diagnostic(path: ArtifactPath, diagnostic: domain::Diagnostic) -> Self {
        Self { path, snapshot: None, diagnostic: Some(diagnostic) }
    }

    /// Returns the repository-relative candidate path.
    #[must_use]
    pub fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the normalized snapshot, when the source read and parse succeeded.
    #[must_use]
    pub fn snapshot(&self) -> Option<&ArtifactSnapshot> {
        self.snapshot.as_ref()
    }

    /// Returns the source diagnostic, when the candidate could not be normalized.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&domain::Diagnostic> {
        self.diagnostic.as_ref()
    }
}
