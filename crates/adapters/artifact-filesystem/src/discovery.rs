use std::fs;
use std::path::{Path, PathBuf};

use application::{ArtifactCandidate, ArtifactIdentitySource, ArtifactSource, ValidationError};
use domain::{ArtifactKind, ArtifactPath, Diagnostic, Severity};

use crate::parse_artifact;

/// Discovers artifacts below a repository root for active and identity validation.
#[derive(Clone, Debug)]
pub struct FilesystemArtifactSource {
    root: PathBuf,
}

impl FilesystemArtifactSource {
    /// Creates a filesystem source rooted at a repository directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the repository root used by this source.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl ArtifactSource for FilesystemArtifactSource {
    fn discover(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        self.discover_from_scope(false)
    }
}

impl ArtifactIdentitySource for FilesystemArtifactSource {
    fn discover_identities(&self) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        self.discover_from_scope(true)
    }
}

impl FilesystemArtifactSource {
    fn discover_from_scope(
        &self,
        include_archive: bool,
    ) -> Result<Vec<ArtifactCandidate>, ValidationError> {
        if !self.root.is_dir() {
            return Err(ValidationError::Discovery(format!(
                "repository root is not a directory: {}",
                self.root.display()
            )));
        }
        let specs = self.root.join("specs");
        if !specs.exists() {
            return Ok(Vec::new());
        }
        if !specs.is_dir() {
            return Err(ValidationError::Discovery(
                "repository specs path is not a directory".to_string(),
            ));
        }

        let mut paths = Vec::new();
        collect_canonical_paths(&self.root, &specs, include_archive, &mut paths)?;
        paths.sort_by(|left, right| left.0.cmp(&right.0));
        paths
            .into_iter()
            .map(|(relative, kind, absolute)| candidate_for_path(&relative, kind, &absolute))
            .collect()
    }
}

type DiscoveredPath = (String, ArtifactKind, PathBuf);

fn collect_canonical_paths(
    root: &Path,
    directory: &Path,
    include_archive: bool,
    paths: &mut Vec<DiscoveredPath>,
) -> Result<(), ValidationError> {
    let entries = fs::read_dir(directory).map_err(|error| {
        ValidationError::Discovery(format!("could not read {}: {error}", directory.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            ValidationError::Discovery(format!(
                "could not inspect {}: {error}",
                directory.display()
            ))
        })?;
        let path = entry.path();
        let relative = path_to_relative(root, &path)?;
        if is_excluded(&relative, include_archive) {
            continue;
        }
        if let Some(kind) = ArtifactKind::from_path(&relative) {
            paths.push((relative, kind, path));
            continue;
        }
        if entry
            .file_type()
            .map_err(|error| {
                ValidationError::Discovery(format!("could not inspect {}: {error}", path.display()))
            })?
            .is_dir()
        {
            collect_canonical_paths(root, &path, include_archive, paths)?;
        }
    }
    Ok(())
}

fn candidate_for_path(
    relative: &str,
    kind: ArtifactKind,
    absolute: &Path,
) -> Result<ArtifactCandidate, ValidationError> {
    let path = ArtifactPath::try_new(relative.to_string()).map_err(|error| {
        ValidationError::Discovery(format!("invalid discovered path `{relative}`: {error}"))
    })?;
    match fs::read_to_string(absolute) {
        Ok(contents) => Ok(ArtifactCandidate::from_snapshot(parse_artifact(path, kind, &contents))),
        Err(error) => {
            let diagnostic = Diagnostic::new(
                path.clone(),
                None,
                format!("ARTIFACT.{}.SOURCE_READ", kind.token().to_ascii_uppercase()),
                Severity::Error,
                format!("artifact could not be read: {error}"),
                "make the canonical artifact readable",
            );
            Ok(ArtifactCandidate::from_diagnostic(path, diagnostic))
        }
    }
}

fn path_to_relative(root: &Path, path: &Path) -> Result<String, ValidationError> {
    let relative = path.strip_prefix(root).map_err(|error| {
        ValidationError::Discovery(format!("could not relativize {}: {error}", path.display()))
    })?;
    relative
        .components()
        .map(|component| {
            component.as_os_str().to_str().map(str::to_owned).ok_or_else(|| {
                ValidationError::Discovery(format!("path is not valid UTF-8: {}", path.display()))
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
}

fn is_excluded(relative: &str, include_archive: bool) -> bool {
    relative
        .split('/')
        .any(|component| component == "templates" || !include_archive && component == "archive")
}
