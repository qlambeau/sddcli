use domain::{ArtifactResult, ValidationReport};

use crate::{ArtifactSource, ValidationError};

/// Validates all candidates supplied by an [`ArtifactSource`].
pub struct Validator<S> {
    source: S,
}

impl<S> Validator<S>
where
    S: ArtifactSource,
{
    /// Creates a validator with an injected source.
    #[must_use]
    pub const fn new(source: S) -> Self {
        Self { source }
    }

    /// Validates every supplied candidate and aggregates all per-artifact findings.
    ///
    /// # Errors
    ///
    /// Returns a discovery error only when the source cannot establish the repository-level
    /// candidate set. Individual candidate failures are retained in the returned report.
    pub fn validate(&self) -> Result<ValidationReport, ValidationError> {
        let candidates = self.source.discover()?;
        let mut results = Vec::with_capacity(candidates.len());

        for candidate in candidates {
            let result = match (candidate.snapshot(), candidate.diagnostic()) {
                (Some(snapshot), _) => ArtifactResult::from_snapshot(snapshot),
                (None, Some(diagnostic)) => {
                    ArtifactResult::from_diagnostic(candidate.path().clone(), diagnostic.clone())
                }
                (None, None) => ArtifactResult::empty(candidate.path().clone()),
            };
            results.push(result);
        }

        Ok(ValidationReport::from_results(results))
    }
}

/// Provides a convenient in-memory source for application tests and callers with normalized data.
#[cfg(any(test, feature = "test-doubles"))]
pub mod fake {
    use super::{ArtifactSource, ValidationError};

    /// Returns a fixed candidate collection without performing I/O.
    #[derive(Clone, Debug)]
    pub struct InMemoryArtifactSource {
        candidates: Vec<crate::ArtifactCandidate>,
        error: Option<String>,
    }

    impl InMemoryArtifactSource {
        /// Creates a source that returns the supplied candidates.
        #[must_use]
        pub fn new(candidates: Vec<crate::ArtifactCandidate>) -> Self {
            Self { candidates, error: None }
        }

        /// Creates a source that returns a repository-level discovery error.
        #[must_use]
        pub fn failing(message: impl Into<String>) -> Self {
            Self { candidates: Vec::new(), error: Some(message.into()) }
        }
    }

    impl ArtifactSource for InMemoryArtifactSource {
        fn discover(&self) -> Result<Vec<crate::ArtifactCandidate>, ValidationError> {
            if let Some(message) = &self.error {
                return Err(ValidationError::Discovery(message.clone()));
            }
            Ok(self.candidates.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use domain::{ArtifactKind, ArtifactPath, ArtifactSnapshot, DocumentSnapshot, Metadata};

    use super::fake::InMemoryArtifactSource;
    use crate::{ArtifactCandidate, ValidationError, Validator};

    fn snapshot(path: &str, kind: ArtifactKind) -> Option<ArtifactSnapshot> {
        Some(ArtifactSnapshot::new(
            ArtifactPath::try_new(path).ok()?,
            kind,
            Metadata::new(),
            DocumentSnapshot::empty(),
        ))
    }

    /// Covers: REQ-001 FR-004 — an empty active set succeeds with no artifacts.
    #[test]
    fn empty_source_returns_successful_empty_report() {
        let result = Validator::new(InMemoryArtifactSource::new(Vec::new())).validate();

        assert!(result.is_ok());
        let Ok(report) = result else { return };
        assert!(report.artifacts().is_empty());
        assert_eq!(report.status(), domain::OverallStatus::Success);
    }

    /// Covers: REQ-001 FR-005 and FR-006 — candidate findings do not abort other candidates.
    #[test]
    fn validates_every_candidate_and_retains_source_diagnostics() {
        let Some(first) = snapshot("specs/first/user-story.md", ArtifactKind::UserStory) else {
            return;
        };
        let Some(path) = ArtifactPath::try_new("specs/second/user-story.md").ok() else {
            return;
        };
        let source_diagnostic = domain::Diagnostic::new(
            path.clone(),
            None,
            "ARTIFACT.USER-STORY.SOURCE",
            domain::Severity::Error,
            "source could not be read",
            "make the file readable",
        );
        let candidates = vec![
            ArtifactCandidate::from_snapshot(first),
            ArtifactCandidate::from_diagnostic(path, source_diagnostic),
        ];
        let result = Validator::new(InMemoryArtifactSource::new(candidates)).validate();

        assert!(result.is_ok());
        let Ok(report) = result else { return };
        assert_eq!(report.artifacts().len(), 2);
        assert!(report.artifacts().iter().all(|artifact| !artifact.violations().is_empty()));
        assert_eq!(report.status(), domain::OverallStatus::Failure);
    }

    /// Covers: REQ-001 FR-004 — repository-level discovery errors are typed and terminal.
    #[test]
    fn returns_typed_error_when_discovery_cannot_be_established() {
        let result = Validator::new(InMemoryArtifactSource::failing("repository root unavailable"))
            .validate();

        assert!(
            matches!(result, Err(ValidationError::Discovery(message)) if message == "repository root unavailable")
        );
    }
}
