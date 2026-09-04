/// A source line retained for diagnostics and marker checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLine {
    line: usize,
    text: String,
}

impl SourceLine {
    /// Creates a source line with its one-based line number.
    #[must_use]
    pub fn new(line: usize, text: impl Into<String>) -> Self {
        Self { line, text: text.into() }
    }

    /// Returns the one-based line number.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the source text without its line ending.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// A Markdown heading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Heading {
    level: u8,
    text: String,
    line: usize,
}

impl Heading {
    /// Creates a normalized heading.
    #[must_use]
    pub fn new(level: u8, text: impl Into<String>, line: usize) -> Self {
        Self { level, text: text.into(), line }
    }

    /// Returns the heading level.
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Returns the heading text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// A Markdown checklist item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChecklistItem {
    checked: bool,
    text: String,
    line: usize,
}

impl ChecklistItem {
    /// Creates a normalized checklist item.
    #[must_use]
    pub fn new(checked: bool, text: impl Into<String>, line: usize) -> Self {
        Self { checked, text: text.into(), line }
    }

    /// Returns whether the item is checked.
    #[must_use]
    pub const fn checked(&self) -> bool {
        self.checked
    }

    /// Returns the item text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// The normalized structure extracted from a Gherkin file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureSnapshot {
    parent: Option<String>,
    status: Option<String>,
    name: Option<String>,
    scenario_count: usize,
    has_given: bool,
    has_when: bool,
    has_then: bool,
}

impl FeatureSnapshot {
    /// Creates a normalized Gherkin structure.
    #[must_use]
    #[allow(
        clippy::similar_names,
        reason = "the three Gherkin step categories are intentionally parallel"
    )]
    pub fn new(
        parent: Option<String>,
        status: Option<String>,
        name: Option<String>,
        scenario_count: usize,
        contains_given_step: bool,
        contains_when_step: bool,
        contains_then_step: bool,
    ) -> Self {
        Self {
            parent,
            status,
            name,
            scenario_count,
            has_given: contains_given_step,
            has_when: contains_when_step,
            has_then: contains_then_step,
        }
    }

    /// Returns the optional workflow parent header.
    #[must_use]
    pub fn parent(&self) -> Option<&str> {
        self.parent.as_deref()
    }

    /// Returns the optional lifecycle status header.
    #[must_use]
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Returns the parsed feature name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the number of scenarios and scenario outlines.
    #[must_use]
    pub const fn scenario_count(&self) -> usize {
        self.scenario_count
    }

    /// Returns whether a Given step was parsed.
    #[must_use]
    pub const fn has_given(&self) -> bool {
        self.has_given
    }

    /// Returns whether a When step was parsed.
    #[must_use]
    pub const fn has_when(&self) -> bool {
        self.has_when
    }

    /// Returns whether a Then step was parsed.
    #[must_use]
    pub const fn has_then(&self) -> bool {
        self.has_then
    }
}

/// Structure extracted from a Markdown or Gherkin artifact.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentSnapshot {
    lines: Vec<SourceLine>,
    headings: Vec<Heading>,
    checklist_items: Vec<ChecklistItem>,
    feature: Option<FeatureSnapshot>,
}

impl DocumentSnapshot {
    /// Creates a normalized document snapshot.
    #[must_use]
    pub fn new(
        lines: Vec<SourceLine>,
        headings: Vec<Heading>,
        checklist_items: Vec<ChecklistItem>,
        feature: Option<FeatureSnapshot>,
    ) -> Self {
        Self { lines, headings, checklist_items, feature }
    }

    /// Returns an empty document snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns source lines in source order.
    #[must_use]
    pub fn lines(&self) -> &[SourceLine] {
        &self.lines
    }

    /// Returns headings in source order.
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    /// Returns checklist items in source order.
    #[must_use]
    pub fn checklist_items(&self) -> &[ChecklistItem] {
        &self.checklist_items
    }

    /// Returns the parsed Gherkin structure, if this is a feature file.
    #[must_use]
    pub fn feature(&self) -> Option<&FeatureSnapshot> {
        self.feature.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers: REQ-001 FR-002 and FR-005 — normalized source structure retains locations and values.
    #[test]
    #[allow(
        clippy::cognitive_complexity,
        reason = "the test covers the complete Markdown boundary contract"
    )]
    fn exposes_markdown_structure() {
        let line = SourceLine::new(3, "## Context");
        let heading = Heading::new(2, "Context", 3);
        let checklist = ChecklistItem::new(true, "Complete", 4);
        let document = DocumentSnapshot::new(
            vec![line.clone()],
            vec![heading.clone()],
            vec![checklist.clone()],
            None,
        );

        assert_eq!(line.line(), 3);
        assert_eq!(line.text(), "## Context");
        assert_eq!(heading.level(), 2);
        assert_eq!(heading.text(), "Context");
        assert_eq!(heading.line(), 3);
        assert!(checklist.checked());
        assert_eq!(checklist.text(), "Complete");
        assert_eq!(checklist.line(), 4);
        assert_eq!(document.lines(), &[line]);
        assert_eq!(document.headings(), &[heading]);
        assert_eq!(document.checklist_items(), &[checklist]);
        assert_eq!(document.feature(), None);
    }

    /// Covers: REQ-001 FR-002 — Gherkin normalized state exposes headers and step presence.
    #[test]
    fn exposes_gherkin_structure() {
        let feature = FeatureSnapshot::new(
            Some("US-001".to_string()),
            Some("approved".to_string()),
            Some("Validate".to_string()),
            2,
            true,
            false,
            true,
        );
        let document = DocumentSnapshot::new(Vec::new(), Vec::new(), Vec::new(), Some(feature));
        let Some(feature) = document.feature() else { return };

        assert_eq!(feature.parent(), Some("US-001"));
        assert_eq!(feature.status(), Some("approved"));
        assert_eq!(feature.name(), Some("Validate"));
        assert_eq!(feature.scenario_count(), 2);
        assert!(feature.has_given());
        assert!(!feature.has_when());
        assert!(feature.has_then());
    }

    /// Covers: REQ-001 FR-004 — an empty normalized document is available for parse failures.
    #[test]
    fn creates_empty_document() {
        let document = DocumentSnapshot::empty();

        assert!(document.lines().is_empty());
        assert!(document.headings().is_empty());
        assert!(document.checklist_items().is_empty());
        assert!(document.feature().is_none());
    }
}
