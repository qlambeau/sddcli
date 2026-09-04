use std::cmp::Ordering;

use crate::ArtifactPath;

/// Identifies the source location of a diagnostic.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Location {
    line: usize,
    column: Option<usize>,
}

impl Location {
    /// Creates a one-based source location.
    #[must_use]
    pub const fn new(line: usize, column: Option<usize>) -> Self {
        Self { line, column }
    }

    /// Returns the one-based line number.
    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    /// Returns the optional one-based column number.
    #[must_use]
    pub const fn column(self) -> Option<usize> {
        self.column
    }
}

/// Identifies a stable validation rule.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuleId(String);

impl RuleId {
    /// Creates a stable rule identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the rule identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Describes diagnostic severity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    /// A non-fatal-looking issue that still fails this validator's strict contract.
    Warning,
    /// A direct validation failure.
    Error,
}

/// Reports one actionable validation violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    path: ArtifactPath,
    location: Option<Location>,
    rule_id: RuleId,
    severity: Severity,
    message: String,
    remediation: String,
}

impl Diagnostic {
    /// Creates an actionable diagnostic.
    #[must_use]
    pub fn new(
        path: ArtifactPath,
        location: Option<Location>,
        rule_id: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            path,
            location,
            rule_id: RuleId::new(rule_id),
            severity,
            message: message.into(),
            remediation: remediation.into(),
        }
    }

    /// Returns the diagnostic path.
    #[must_use]
    pub fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the optional source location.
    #[must_use]
    pub const fn location(&self) -> Option<Location> {
        self.location
    }

    /// Returns the stable rule identifier.
    #[must_use]
    pub fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    /// Returns the diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the actionable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns remediation guidance.
    #[must_use]
    pub fn remediation(&self) -> &str {
        &self.remediation
    }

    pub(crate) fn stable_cmp(&self, other: &Self) -> Ordering {
        self.rule_id
            .cmp(&other.rule_id)
            .then_with(|| self.location.cmp(&other.location))
            .then_with(|| self.severity.cmp(&other.severity))
            .then_with(|| self.message.cmp(&other.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> Option<ArtifactPath> {
        ArtifactPath::try_new("specs/feature/user-story.md").ok()
    }

    /// Covers: REQ-001 FR-005 and FR-008 — diagnostics retain all actionable fields and ordering data.
    #[test]
    #[allow(
        clippy::cognitive_complexity,
        reason = "the test covers the complete diagnostic accessor contract"
    )]
    fn exposes_diagnostic_context_and_stable_comparison() {
        let Some(path) = path() else { return };
        let first = Diagnostic::new(
            path.clone(),
            Some(Location::new(2, Some(4))),
            "ARTIFACT.USER-STORY.A_RULE",
            Severity::Warning,
            "message",
            "remediation",
        );
        let second = Diagnostic::new(
            path.clone(),
            Some(Location::new(3, None)),
            "ARTIFACT.USER-STORY.B_RULE",
            Severity::Error,
            "message",
            "remediation",
        );

        assert_eq!(first.path().as_str(), path.as_str());
        assert_eq!(first.location(), Some(Location::new(2, Some(4))));
        assert_eq!(first.location().map(Location::line), Some(2));
        assert_eq!(first.location().and_then(Location::column), Some(4));
        assert_eq!(first.rule_id().as_str(), "ARTIFACT.USER-STORY.A_RULE");
        assert_eq!(first.severity(), Severity::Warning);
        assert_eq!(first.message(), "message");
        assert_eq!(first.remediation(), "remediation");
        assert!(first.stable_cmp(&second).is_lt());
        assert!(RuleId::new("B").cmp(&RuleId::new("A")).is_gt());
    }

    /// Covers: REQ-001 FR-008 — warning and error severities are distinct ordered values.
    #[test]
    fn orders_severity_values() {
        assert!(Severity::Warning < Severity::Error);
        assert_eq!(Location::new(1, None).column(), None);
        assert_eq!(Location::new(1, None).line(), 1);
    }
}
