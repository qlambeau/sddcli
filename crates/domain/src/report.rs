use std::cmp::Ordering;

use crate::{
    ArtifactId, ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, validate_artifact,
};

/// Status assigned to one artifact result.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArtifactStatus {
    /// The artifact satisfies all applicable rules.
    Ok,
    /// The artifact has one or more violations.
    Diagnostic,
}

/// Overall validation status.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OverallStatus {
    /// No artifact has a diagnostic status.
    Success,
    /// At least one artifact has a diagnostic status.
    Failure,
}

impl OverallStatus {
    /// Calculates the overall result from artifact results.
    #[must_use]
    pub fn from_results(results: &[ArtifactResult]) -> Self {
        if results.iter().any(|result| result.status == ArtifactStatus::Diagnostic) {
            Self::Failure
        } else {
            Self::Success
        }
    }
}

/// Validation findings for one artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactResult {
    path: ArtifactPath,
    kind: Option<ArtifactKind>,
    id: Option<ArtifactId>,
    status: ArtifactStatus,
    violations: Vec<Diagnostic>,
}

impl ArtifactResult {
    /// Validates a normalized snapshot and creates its result.
    #[must_use]
    pub fn from_snapshot(snapshot: &ArtifactSnapshot) -> Self {
        let mut violations = validate_artifact(snapshot);
        violations.sort_by(Diagnostic::stable_cmp);
        Self {
            path: snapshot.path().clone(),
            kind: Some(snapshot.kind()),
            id: snapshot.id().cloned(),
            status: status_for(&violations),
            violations,
        }
    }

    /// Creates a result for a source failure that has no normalized snapshot.
    #[must_use]
    pub fn from_diagnostic(path: ArtifactPath, diagnostic: Diagnostic) -> Self {
        let kind = ArtifactKind::from_path(path.as_str());
        let id = kind.and_then(|kind| {
            path.as_str()
                .rsplit('/')
                .next()
                .and_then(|file| file.strip_suffix(".md"))
                .and_then(|value| ArtifactId::try_new(kind, value).ok())
        });
        Self { path, kind, id, status: ArtifactStatus::Diagnostic, violations: vec![diagnostic] }
    }

    /// Returns an internal diagnostic result for an invalid empty candidate.
    #[must_use]
    pub fn empty(path: ArtifactPath) -> Self {
        let diagnostic = Diagnostic::new(
            path.clone(),
            None,
            "ARTIFACT.INTERNAL.EMPTY_CANDIDATE",
            crate::Severity::Error,
            "the candidate contained neither a snapshot nor a diagnostic",
            "provide a normalized snapshot or a source diagnostic",
        );
        Self::from_diagnostic(path, diagnostic)
    }

    /// Returns the repository-relative path.
    #[must_use]
    pub fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the recognized kind, when available.
    #[must_use]
    pub const fn kind(&self) -> Option<ArtifactKind> {
        self.kind
    }

    /// Returns the recognized identifier, when available.
    #[must_use]
    pub fn id(&self) -> Option<&ArtifactId> {
        self.id.as_ref()
    }

    /// Returns the artifact status.
    #[must_use]
    pub const fn status(&self) -> ArtifactStatus {
        self.status
    }

    /// Returns all violations in stable order.
    #[must_use]
    pub fn violations(&self) -> &[Diagnostic] {
        &self.violations
    }
}

fn status_for(violations: &[Diagnostic]) -> ArtifactStatus {
    if violations.is_empty() { ArtifactStatus::Ok } else { ArtifactStatus::Diagnostic }
}

/// The complete ordered validation report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    status: OverallStatus,
    artifacts: Vec<ArtifactResult>,
}

impl ValidationReport {
    /// Creates a report and applies the fixed artifact ordering.
    #[must_use]
    pub fn from_results(mut artifacts: Vec<ArtifactResult>) -> Self {
        artifacts.sort_by(compare_results);
        Self { status: OverallStatus::from_results(&artifacts), artifacts }
    }

    /// Returns the overall validation result.
    #[must_use]
    pub const fn status(&self) -> OverallStatus {
        self.status
    }

    /// Returns ordered per-artifact results.
    #[must_use]
    pub fn artifacts(&self) -> &[ArtifactResult] {
        &self.artifacts
    }
}

fn compare_results(left: &ArtifactResult, right: &ArtifactResult) -> Ordering {
    let left_order = left.kind.map_or(ArtifactKind::ALL.len(), kind_order);
    let right_order = right.kind.map_or(ArtifactKind::ALL.len(), kind_order);
    left_order
        .cmp(&right_order)
        .then_with(|| compare_optional_ids(left.id.as_ref(), right.id.as_ref()))
        .then_with(|| left.path.cmp(&right.path))
}

fn kind_order(kind: ArtifactKind) -> usize {
    ArtifactKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .unwrap_or(ArtifactKind::ALL.len())
}

fn compare_optional_ids(left: Option<&ArtifactId>, right: Option<&ArtifactId>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentSnapshot, Metadata, Severity};

    fn path(value: &str) -> Option<ArtifactPath> {
        ArtifactPath::try_new(value).ok()
    }

    /// Covers: REQ-001 FR-004 and FR-007 — report and artifact results expose ordered statuses and identity.
    #[test]
    fn exposes_result_identity_and_status() {
        let Some(path) = path("specs/prds/PRD-001.md") else { return };
        let mut metadata = Metadata::new();
        metadata.insert_scalar("id", "PRD-001");
        let snapshot = ArtifactSnapshot::new(
            path.clone(),
            ArtifactKind::Prd,
            metadata,
            DocumentSnapshot::empty(),
        );
        let result = ArtifactResult::from_snapshot(&snapshot);
        let report = ValidationReport::from_results(vec![result.clone()]);

        assert_eq!(result.path().as_str(), path.as_str());
        assert_eq!(result.kind(), Some(ArtifactKind::Prd));
        assert_eq!(result.id().map(ArtifactId::as_str), Some("PRD-001"));
        assert_eq!(result.status(), ArtifactStatus::Diagnostic);
        assert!(!result.violations().is_empty());
        assert_eq!(report.status(), OverallStatus::Failure);
        assert_eq!(report.artifacts(), &[result]);
        assert_eq!(OverallStatus::from_results(&[]), OverallStatus::Success);
    }

    /// Covers: REQ-001 FR-006 and FR-007 — path-only findings remain identifiable and sort after typed results.
    #[test]
    fn creates_path_only_diagnostic_result() {
        let Some(unknown_path) = path("specs/unrecognized.md") else { return };
        let diagnostic = Diagnostic::new(
            unknown_path.clone(),
            None,
            "ARTIFACT.UNKNOWN.PARSE",
            Severity::Error,
            "cannot parse",
            "repair the file",
        );
        let result = ArtifactResult::from_diagnostic(unknown_path.clone(), diagnostic);
        let Some(identified_path) = path("specs/prds/PRD-001.md") else { return };
        let mut metadata = Metadata::new();
        metadata.insert_scalar("id", "PRD-001");
        let identified_snapshot = ArtifactSnapshot::new(
            identified_path,
            ArtifactKind::Prd,
            metadata,
            DocumentSnapshot::empty(),
        );
        let identified = ArtifactResult::from_snapshot(&identified_snapshot);
        let report = ValidationReport::from_results(vec![result.clone(), identified]);

        assert_eq!(result.path().as_str(), unknown_path.as_str());
        assert_eq!(result.kind(), None);
        assert_eq!(result.id(), None);
        assert_eq!(result.status(), ArtifactStatus::Diagnostic);
        assert_eq!(report.artifacts().len(), 2);
        assert_eq!(report.artifacts().get(1), Some(&result));
    }

    /// Covers: REQ-001 FR-005 — a malformed candidate produces an actionable internal finding.
    #[test]
    fn creates_internal_candidate_diagnostic() {
        let Some(path) = path("specs/unrecognized.md") else { return };
        let result = ArtifactResult::empty(path);

        assert_eq!(result.violations().len(), 1);
        let Some(diagnostic) = result.violations().first() else { return };
        assert_eq!(diagnostic.severity(), Severity::Error);
    }
}
