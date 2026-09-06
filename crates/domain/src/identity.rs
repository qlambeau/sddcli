use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::{ArtifactId, ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, Severity};

const DUPLICATE_ID_RULE: &str = "ARTIFACT.IDENTITY.DUPLICATE_ID";
const PATH_IDENTITY_RULE: &str = "ARTIFACT.IDENTITY.PATH_IDENTITY";

/// Evaluates recognized artifact identities across one repository snapshot.
///
/// The evaluation only considers snapshots with recognized identifiers. Parser
/// diagnostics and unrecognized frontmatter remain the responsibility of the
/// structural validation path.
#[must_use]
pub fn validate_identities(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    let mut diagnostics = duplicate_id_diagnostics(snapshots);
    diagnostics.extend(path_identity_diagnostics(snapshots));
    diagnostics.sort_by(identity_diagnostic_cmp);
    diagnostics
}

fn duplicate_id_diagnostics(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    let mut paths_by_id: BTreeMap<ArtifactId, BTreeSet<ArtifactPath>> = BTreeMap::new();
    for snapshot in snapshots {
        let Some(id) = snapshot.id() else { continue };
        paths_by_id.entry(id.clone()).or_default().insert(snapshot.path().clone());
    }

    let mut diagnostics = Vec::new();
    for (id, paths) in paths_by_id {
        if paths.len() < 2 {
            continue;
        }
        let path_list = paths.iter().map(ArtifactPath::as_str).collect::<Vec<_>>().join(", ");
        for path in paths {
            diagnostics.push(Diagnostic::new(
                path,
                None,
                DUPLICATE_ID_RULE,
                Severity::Error,
                format!(
                    "artifact identifier `{}` is used by multiple artifacts: {path_list}",
                    id.as_str()
                ),
                "assign a unique identifier to this artifact and update affected references",
            ));
        }
    }
    diagnostics
}

fn path_identity_diagnostics(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    snapshots
        .iter()
        .filter_map(|snapshot| {
            let frontmatter_id = snapshot.id()?;
            let filename_id = path_encoded_id(snapshot.kind(), snapshot.path())?;
            if frontmatter_id == &filename_id {
                return None;
            }
            Some(Diagnostic::new(
                snapshot.path().clone(),
                None,
                PATH_IDENTITY_RULE,
                Severity::Error,
                format!(
                    "filename identity `{}` does not match frontmatter identity `{}`",
                    filename_id.as_str(),
                    frontmatter_id.as_str()
                ),
                format!(
                    "rename the file to `{}` or update its frontmatter `id` to `{}`",
                    filename_id.as_str(),
                    filename_id.as_str()
                ),
            ))
        })
        .collect()
}

fn path_encoded_id(kind: ArtifactKind, path: &ArtifactPath) -> Option<ArtifactId> {
    if !matches!(kind, ArtifactKind::Prd | ArtifactKind::Epic | ArtifactKind::Adr) {
        return None;
    }
    let filename = path.as_str().rsplit('/').next()?.strip_suffix(".md")?;
    ArtifactId::try_new(kind, filename).ok()
}

fn identity_diagnostic_cmp(left: &Diagnostic, right: &Diagnostic) -> Ordering {
    left.rule_id()
        .cmp(right.rule_id())
        .then_with(|| left.path().cmp(right.path()))
        .then_with(|| left.stable_cmp(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers: REQ-002 FR-002 — path identity extraction recognizes only encoded PRD, epic, and ADR names.
    #[test]
    fn extracts_only_path_encoded_identity_kinds() {
        let Ok(prd_path) = ArtifactPath::try_new("specs/prds/PRD-001.md") else { return };
        let Ok(story_path) = ArtifactPath::try_new("specs/story/user-story.md") else { return };

        assert_eq!(
            path_encoded_id(ArtifactKind::Prd, &prd_path).map(|id| id.as_str().to_owned()),
            Some("PRD-001".to_owned())
        );
        assert_eq!(path_encoded_id(ArtifactKind::UserStory, &story_path), None);
    }
}
