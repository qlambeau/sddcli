use domain::{ArtifactPath, ArtifactSnapshot, PromotionPlan, PromotionTimestamp};

use crate::{PromotionError, ValidationError};

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

/// Loads and commits one artifact promotion using expected-source protection.
pub trait ArtifactPromotionStore {
    /// Loads the target source exactly as it exists before promotion preflight.
    ///
    /// # Errors
    ///
    /// Returns a typed promotion error when the target source cannot be loaded.
    fn load(&self, path: &ArtifactPath) -> Result<SourceArtifact, PromotionError>;

    /// Commits one accepted promotion plan if the current source still matches the expected source.
    ///
    /// # Errors
    ///
    /// Returns a typed promotion error for source conflicts, unsupported formats, or write failures.
    fn commit(
        &mut self,
        source: &SourceArtifact,
        plan: &PromotionPlan,
    ) -> Result<SourceArtifact, PromotionError>;
}

/// Supplies UTC Unix seconds for real promotion transitions.
pub trait Clock {
    /// Returns the current UTC Unix-second timestamp.
    ///
    /// # Errors
    ///
    /// Returns a typed promotion error when a timestamp cannot be supplied.
    fn now(&self) -> Result<PromotionTimestamp, PromotionError>;
}

/// Contains an artifact's expected source for protected promotion commits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceArtifact {
    path: ArtifactPath,
    contents: String,
}

impl SourceArtifact {
    /// Creates a captured source artifact.
    #[must_use]
    pub fn new(path: ArtifactPath, contents: impl Into<String>) -> Self {
        Self { path, contents: contents.into() }
    }

    /// Returns the captured source path.
    #[must_use]
    pub const fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the captured source bytes as UTF-8 text.
    #[must_use]
    pub fn contents(&self) -> &str {
        &self.contents
    }
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

/// Provides deterministic promotion port doubles for tests and callers with normalized data.
#[cfg(any(test, feature = "test-doubles"))]
#[allow(
    unreachable_pub,
    reason = "test doubles are re-exported from the crate root under the same cfg"
)]
pub mod fake {
    use std::cell::Cell;

    use domain::{PromotionPlan, PromotionTimestamp};

    use super::{ArtifactPath, ArtifactPromotionStore, Clock, PromotionError, SourceArtifact};

    /// In-memory promotion store that captures commits without performing I/O.
    #[derive(Clone, Debug)]
    pub struct InMemoryPromotionStore {
        source: SourceArtifact,
        commit_count: usize,
        conflict: bool,
    }

    impl InMemoryPromotionStore {
        /// Creates a store returning the supplied source.
        #[must_use]
        pub const fn new(source: SourceArtifact) -> Self {
            Self { source, commit_count: 0, conflict: false }
        }

        /// Creates a store that reports a conflict on commit.
        #[must_use]
        pub const fn conflicting(source: SourceArtifact) -> Self {
            Self { source, commit_count: 0, conflict: true }
        }

        /// Returns the number of commit attempts.
        #[must_use]
        pub const fn commit_count(&self) -> usize {
            self.commit_count
        }
    }

    impl ArtifactPromotionStore for InMemoryPromotionStore {
        fn load(&self, path: &ArtifactPath) -> Result<SourceArtifact, PromotionError> {
            if self.source.path() == path {
                Ok(self.source.clone())
            } else {
                Err(PromotionError::TargetNotFound(path.clone()))
            }
        }

        fn commit(
            &mut self,
            source: &SourceArtifact,
            _plan: &PromotionPlan,
        ) -> Result<SourceArtifact, PromotionError> {
            self.commit_count += 1;
            if self.conflict {
                return Err(PromotionError::Conflict(source.path().clone()));
            }
            self.source = source.clone();
            Ok(self.source.clone())
        }
    }

    /// Deterministic clock for promotion tests.
    #[derive(Debug)]
    pub struct FixedClock {
        timestamp: PromotionTimestamp,
        error: Option<String>,
        calls: Cell<usize>,
    }

    impl FixedClock {
        /// Creates a clock returning a fixed timestamp.
        #[must_use]
        pub const fn new(timestamp: PromotionTimestamp) -> Self {
            Self { timestamp, error: None, calls: Cell::new(0) }
        }

        /// Creates a clock that fails if used.
        #[must_use]
        pub fn failing(message: impl Into<String>) -> Self {
            Self {
                timestamp: PromotionTimestamp::from_unix_seconds(0),
                error: Some(message.into()),
                calls: Cell::new(0),
            }
        }

        /// Returns how many timestamps were requested.
        #[must_use]
        pub fn calls(&self) -> usize {
            self.calls.get()
        }
    }

    impl Clock for FixedClock {
        fn now(&self) -> Result<PromotionTimestamp, PromotionError> {
            self.calls.set(self.calls.get() + 1);
            if let Some(message) = &self.error {
                return Err(PromotionError::Clock(message.clone()));
            }
            Ok(self.timestamp)
        }
    }
}
