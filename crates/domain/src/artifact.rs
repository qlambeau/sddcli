use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{Diagnostic, DocumentSnapshot};

/// Identifies one of the artifact formats supported by the validator.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArtifactKind {
    /// A product requirements document.
    Prd,
    /// An epic brief.
    Epic,
    /// A user story.
    UserStory,
    /// An executable Gherkin scenario file.
    Gherkin,
    /// A feature requirements document.
    Requirements,
    /// A feature design document.
    Design,
    /// An architecture decision record.
    Adr,
    /// An implementation task document.
    Task,
}

impl ArtifactKind {
    /// Returns artifact kinds in the required report order.
    pub const ALL: [Self; 8] = [
        Self::Prd,
        Self::Epic,
        Self::UserStory,
        Self::Gherkin,
        Self::Requirements,
        Self::Design,
        Self::Adr,
        Self::Task,
    ];

    /// Returns the stable lower-case type token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Prd => "prd",
            Self::Epic => "epic",
            Self::UserStory => "user-story",
            Self::Gherkin => "gherkin",
            Self::Requirements => "requirements",
            Self::Design => "design",
            Self::Adr => "adr",
            Self::Task => "task",
        }
    }

    /// Returns the required frontmatter type value, when the artifact has one.
    #[must_use]
    pub const fn expected_type(self) -> Option<&'static str> {
        match self {
            Self::Prd => Some("product-requirements"),
            Self::Epic => Some("epic-brief"),
            Self::UserStory => Some("user-story"),
            Self::Gherkin => None,
            Self::Requirements => Some("feature-requirements"),
            Self::Design => Some("feature-design"),
            Self::Adr => Some("architecture-decision-record"),
            Self::Task => Some("implementation-tasks"),
        }
    }

    /// Returns the identifier prefix required by this artifact kind.
    #[must_use]
    pub const fn id_prefix(self) -> Option<&'static str> {
        match self {
            Self::Prd => Some("PRD"),
            Self::Epic => Some("EPIC"),
            Self::UserStory => Some("US"),
            Self::Gherkin => None,
            Self::Requirements => Some("REQ"),
            Self::Design => Some("DES"),
            Self::Adr => Some("ADR"),
            Self::Task => Some("TASK"),
        }
    }

    /// Identifies the artifact kind from a canonical repository-relative path.
    #[must_use]
    pub fn from_path(path: &str) -> Option<Self> {
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() == 3 && segments.first() == Some(&"specs") {
            let packet_kind = packet_kind(segments.get(2).copied());
            if packet_kind.is_some() {
                return packet_kind;
            }
        }
        if segments.len() == 4
            && segments.first() == Some(&"specs")
            && segments.get(1) == Some(&"archive")
            && segments.get(2).is_some_and(|directory| valid_archive_directory(directory))
        {
            return packet_kind(segments.get(3).copied());
        }
        if segments.len() == 3
            && segments.first() == Some(&"specs")
            && segments.get(1) == Some(&"adr")
            && segments.get(2).is_some_and(|file| valid_numbered_name(file, "ADR"))
        {
            return Some(Self::Adr);
        }
        if segments.len() == 3
            && segments.first() == Some(&"specs")
            && segments.get(1) == Some(&"prds")
            && segments.get(2).is_some_and(|file| valid_numbered_name(file, "PRD"))
        {
            return Some(Self::Prd);
        }
        if segments.len() == 4
            && segments.first() == Some(&"specs")
            && segments.get(1) == Some(&"prds")
            && segments.get(2).is_some_and(|directory| valid_numbered_directory(directory, "PRD-"))
            && segments.get(3).is_some_and(|file| valid_numbered_name(file, "EPIC"))
        {
            return Some(Self::Epic);
        }
        None
    }
}

fn packet_kind(file: Option<&str>) -> Option<ArtifactKind> {
    match file {
        Some("user-story.md") => Some(ArtifactKind::UserStory),
        Some("scenarios.feature") => Some(ArtifactKind::Gherkin),
        Some("requirements.md") => Some(ArtifactKind::Requirements),
        Some("design.md") => Some(ArtifactKind::Design),
        Some("tasks.md") => Some(ArtifactKind::Task),
        _ => None,
    }
}

fn valid_numbered_name(value: &str, prefix: &str) -> bool {
    let Some(number) = value.strip_prefix(prefix).and_then(|value| value.strip_prefix('-')) else {
        return false;
    };
    let Some(number) = number.strip_suffix(".md") else {
        return false;
    };
    number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_numbered_directory(value: &str, prefix: &str) -> bool {
    let Some(number) = value.strip_prefix(prefix) else {
        return false;
    };
    let Some(number) = number.strip_suffix("-epics") else {
        return false;
    };
    number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_archive_directory(value: &str) -> bool {
    value.len() > 4
        && value.as_bytes().get(3) == Some(&b'-')
        && value.as_bytes().get(0..3).is_some_and(|number| number.iter().all(u8::is_ascii_digit))
}

/// Identifies a repository-relative artifact path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactPath(String);

impl ArtifactPath {
    /// Creates a validated repository-relative path.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty, absolute, traversing, or platform-ambiguous path.
    pub fn try_new(value: impl Into<String>) -> Result<Self, PathError> {
        let value = value.into();
        if value.is_empty() {
            return Err(PathError::Empty);
        }
        if value.starts_with('/') || value.contains('\\') {
            return Err(PathError::NotRepositoryRelative);
        }
        if value.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
            return Err(PathError::NotRepositoryRelative);
        }
        Ok(Self(value))
    }

    /// Returns the repository-relative slash-separated path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explains why an artifact path could not be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathError {
    /// The supplied path had no components.
    Empty,
    /// The supplied path was not repository-relative.
    NotRepositoryRelative,
}

impl Display for PathError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("artifact path is empty"),
            Self::NotRepositoryRelative => {
                formatter.write_str("artifact path is not repository-relative")
            }
        }
    }
}

impl Error for PathError {}

/// Identifies a valid numbered artifact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactId(String);

impl ArtifactId {
    /// Creates an identifier with the prefix required by its artifact kind.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier does not use the expected three-digit form.
    pub fn try_new(kind: ArtifactKind, value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        let Some(prefix) = kind.id_prefix() else {
            return Err(IdentifierError::UnsupportedKind);
        };
        if !valid_identifier(&value, prefix) {
            return Err(IdentifierError::Invalid { expected_prefix: prefix });
        }
        Ok(Self(value))
    }

    /// Returns the identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_identifier(value: &str, prefix: &str) -> bool {
    let Some(number) = value.strip_prefix(prefix) else {
        return false;
    };
    let Some(number) = number.strip_prefix('-') else {
        return false;
    };
    number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

/// Explains why an artifact identifier was rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    /// The artifact kind has no identifier.
    UnsupportedKind,
    /// The identifier had the wrong prefix or numeric width.
    Invalid {
        /// The expected identifier prefix.
        expected_prefix: &'static str,
    },
}

impl Display for IdentifierError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedKind => {
                formatter.write_str("artifact kind does not use an identifier")
            }
            Self::Invalid { expected_prefix } => {
                write!(formatter, "identifier must use {expected_prefix}-NNN")
            }
        }
    }
}

impl Error for IdentifierError {}

/// A normalized frontmatter field value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataValue {
    /// A scalar YAML value represented as text.
    Scalar(String),
    /// A YAML sequence of scalar values represented as text.
    Sequence(Vec<String>),
    /// A YAML null value.
    Null,
    /// A YAML mapping or unsupported nested value.
    Mapping,
}

impl MetadataValue {
    /// Creates a scalar metadata value.
    #[must_use]
    pub fn scalar(value: impl Into<String>) -> Self {
        Self::Scalar(value.into())
    }

    /// Creates a sequence metadata value.
    #[must_use]
    pub fn sequence<I, V>(values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<String>,
    {
        Self::Sequence(values.into_iter().map(Into::into).collect())
    }
}

/// Normalized frontmatter fields without a serialization dependency.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    fields: BTreeMap<String, MetadataValue>,
}

impl Metadata {
    /// Creates empty metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a normalized metadata value.
    pub fn insert(&mut self, name: impl Into<String>, value: MetadataValue) {
        self.fields.insert(name.into(), value);
    }

    /// Inserts a scalar field.
    pub fn insert_scalar(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.insert(name, MetadataValue::scalar(value));
    }

    /// Inserts a sequence field.
    pub fn insert_sequence<I, V>(&mut self, name: impl Into<String>, values: I)
    where
        I: IntoIterator<Item = V>,
        V: Into<String>,
    {
        self.insert(name, MetadataValue::sequence(values));
    }

    /// Returns a field by its YAML name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&MetadataValue> {
        self.fields.get(name)
    }

    /// Returns all fields in stable name order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &MetadataValue)> {
        self.fields.iter().map(|(name, value)| (name.as_str(), value))
    }
}

/// A normalized artifact ready for pure validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactSnapshot {
    path: ArtifactPath,
    kind: ArtifactKind,
    id: Option<ArtifactId>,
    metadata: Metadata,
    document: DocumentSnapshot,
    parser_diagnostics: Vec<Diagnostic>,
}

impl ArtifactSnapshot {
    /// Creates a normalized artifact snapshot.
    #[must_use]
    pub fn new(
        path: ArtifactPath,
        kind: ArtifactKind,
        metadata: Metadata,
        document: DocumentSnapshot,
    ) -> Self {
        let id = metadata.get("id").and_then(|value| match value {
            MetadataValue::Scalar(value) => ArtifactId::try_new(kind, value.clone()).ok(),
            _ => None,
        });
        Self { path, kind, id, metadata, document, parser_diagnostics: Vec::new() }
    }

    /// Adds diagnostics produced while parsing the source document.
    #[must_use]
    pub fn with_parser_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.parser_diagnostics = diagnostics;
        self
    }

    /// Returns the repository-relative path.
    #[must_use]
    pub fn path(&self) -> &ArtifactPath {
        &self.path
    }

    /// Returns the canonical artifact kind.
    #[must_use]
    pub const fn kind(&self) -> ArtifactKind {
        self.kind
    }

    /// Returns the recognized identifier, if valid.
    #[must_use]
    pub fn id(&self) -> Option<&ArtifactId> {
        self.id.as_ref()
    }

    /// Returns normalized frontmatter.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Returns normalized document structure.
    #[must_use]
    pub const fn document(&self) -> &DocumentSnapshot {
        &self.document
    }

    /// Returns parser findings attached to this candidate.
    #[must_use]
    pub fn parser_diagnostics(&self) -> &[Diagnostic] {
        &self.parser_diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(value: &str) -> Option<ArtifactPath> {
        ArtifactPath::try_new(value).ok()
    }

    /// Covers: REQ-001 FR-001 and FR-007 — kind tokens and canonical paths map deterministically.
    #[test]
    fn identifies_every_kind_and_canonical_path() {
        let paths = [
            ("specs/user/user-story.md", ArtifactKind::UserStory),
            ("specs/user/scenarios.feature", ArtifactKind::Gherkin),
            ("specs/user/requirements.md", ArtifactKind::Requirements),
            ("specs/user/design.md", ArtifactKind::Design),
            ("specs/user/tasks.md", ArtifactKind::Task),
            ("specs/adr/ADR-001.md", ArtifactKind::Adr),
            ("specs/prds/PRD-001.md", ArtifactKind::Prd),
            ("specs/prds/PRD-001-epics/EPIC-001.md", ArtifactKind::Epic),
        ];

        assert!(
            paths.iter().all(|(value, expected)| ArtifactKind::from_path(value) == Some(*expected))
        );
        assert_eq!(
            ArtifactKind::from_path("specs/archive/001-old/user-story.md"),
            Some(ArtifactKind::UserStory)
        );
        assert_eq!(ArtifactKind::from_path("specs/archive/old/user-story.md"), None);
        assert_eq!(ArtifactKind::from_path("specs/not-an-artifact.md"), None);
        assert_eq!(ArtifactKind::from_path("specs/prds/PRD-01.md"), None);
        assert_eq!(ArtifactKind::from_path("specs/adr/ADR-01.md"), None);
        assert_eq!(ArtifactKind::from_path("specs/prds/PRD-01-epics/EPIC-001.md"), None);
        assert_eq!(ArtifactKind::from_path("specs/prds/PRD-001-epics/EPIC-01.md"), None);
    }

    /// Covers: REQ-001 FR-002 — identifier formats reject wrong prefixes and widths.
    #[test]
    #[allow(
        clippy::cognitive_complexity,
        reason = "the test covers all invalid invariant branches"
    )]
    fn validates_identifier_and_path_invariants() {
        let Ok(identifier) = ArtifactId::try_new(ArtifactKind::Prd, "PRD-001") else { return };

        assert_eq!(identifier.as_str(), "PRD-001");
        assert!(matches!(
            ArtifactId::try_new(ArtifactKind::Prd, "REQ-001"),
            Err(IdentifierError::Invalid { .. })
        ));
        assert!(matches!(
            ArtifactId::try_new(ArtifactKind::Prd, "PRD-01"),
            Err(IdentifierError::Invalid { .. })
        ));
        assert!(matches!(
            ArtifactId::try_new(ArtifactKind::Gherkin, "GHERKIN-001"),
            Err(IdentifierError::UnsupportedKind)
        ));
        assert_eq!(PathError::Empty.to_string(), "artifact path is empty");
        assert_eq!(
            PathError::NotRepositoryRelative.to_string(),
            "artifact path is not repository-relative"
        );
        assert!(matches!(ArtifactPath::try_new(""), Err(PathError::Empty)));
        assert!(matches!(
            ArtifactPath::try_new("/specs/a.md"),
            Err(PathError::NotRepositoryRelative)
        ));
        assert!(matches!(
            ArtifactPath::try_new("specs/../a.md"),
            Err(PathError::NotRepositoryRelative)
        ));
        assert!(matches!(
            ArtifactPath::try_new("specs\\a.md"),
            Err(PathError::NotRepositoryRelative)
        ));
        assert_eq!(
            IdentifierError::UnsupportedKind.to_string(),
            "artifact kind does not use an identifier"
        );
        assert_eq!(
            IdentifierError::Invalid { expected_prefix: "PRD" }.to_string(),
            "identifier must use PRD-NNN"
        );
    }

    /// Covers: REQ-001 FR-002 — normalized metadata and snapshots expose stable boundary data.
    #[test]
    #[allow(
        clippy::cognitive_complexity,
        reason = "the test covers the complete boundary accessor contract"
    )]
    fn stores_metadata_and_parser_findings() {
        let Some(path) = path("specs/user/user-story.md") else { return };
        let mut metadata = Metadata::new();
        metadata.insert_scalar("title", "Title");
        metadata.insert_sequence("related", ["ADR-001"]);
        metadata.insert("nested", MetadataValue::Mapping);
        let document = DocumentSnapshot::empty();
        let finding = Diagnostic::new(
            path.clone(),
            None,
            "ARTIFACT.USER-STORY.TEST",
            crate::Severity::Warning,
            "finding",
            "fix finding",
        );
        let snapshot =
            ArtifactSnapshot::new(path.clone(), ArtifactKind::UserStory, metadata, document)
                .with_parser_diagnostics(vec![finding]);

        assert_eq!(snapshot.path().as_str(), path.as_str());
        assert_eq!(snapshot.kind(), ArtifactKind::UserStory);
        assert_eq!(snapshot.id(), None);
        assert!(
            matches!(snapshot.metadata().get("title"), Some(MetadataValue::Scalar(value)) if value == "Title")
        );
        assert!(
            matches!(snapshot.metadata().get("related"), Some(MetadataValue::Sequence(values)) if values == &["ADR-001"])
        );
        assert_eq!(snapshot.metadata().iter().count(), 3);
        assert_eq!(snapshot.document(), &DocumentSnapshot::empty());
        assert_eq!(snapshot.parser_diagnostics().len(), 1);
        assert_eq!(MetadataValue::scalar("value"), MetadataValue::Scalar("value".to_string()));
        assert_eq!(
            MetadataValue::sequence(["one", "two"]),
            MetadataValue::Sequence(vec!["one".to_string(), "two".to_string()])
        );
    }

    /// Covers: REQ-001 FR-007 — each kind has stable output metadata.
    #[test]
    fn exposes_kind_contracts() {
        assert_eq!(ArtifactKind::ALL.len(), 8);
        assert_eq!(ArtifactKind::Prd.token(), "prd");
        assert_eq!(ArtifactKind::Gherkin.expected_type(), None);
        assert_eq!(ArtifactKind::Gherkin.id_prefix(), None);
        assert_eq!(ArtifactKind::Task.expected_type(), Some("implementation-tasks"));
        assert_eq!(ArtifactKind::Adr.id_prefix(), Some("ADR"));
    }
}
