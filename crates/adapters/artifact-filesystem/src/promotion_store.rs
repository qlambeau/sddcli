use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use application::{ArtifactPromotionStore, Clock, PromotionError, SourceArtifact};
use domain::{ArtifactKind, ArtifactPath, PromotionPlan, PromotionTimestamp};

/// Loads and atomically patches artifact promotion metadata on the local filesystem.
#[derive(Clone, Debug)]
pub struct FilesystemPromotionStore {
    root: PathBuf,
}

impl FilesystemPromotionStore {
    /// Creates a promotion store rooted at a repository directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the repository root used by this store.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn absolute(&self, path: &ArtifactPath) -> PathBuf {
        self.root.join(path.as_str())
    }
}

impl ArtifactPromotionStore for FilesystemPromotionStore {
    fn load(&self, path: &ArtifactPath) -> Result<SourceArtifact, PromotionError> {
        let absolute = self.absolute(path);
        let contents = fs::read_to_string(&absolute)
            .map_err(|error| PromotionError::Source(format!("{}: {error}", absolute.display())))?;
        Ok(SourceArtifact::new(path.clone(), contents))
    }

    fn commit(
        &mut self,
        source: &SourceArtifact,
        plan: &PromotionPlan,
    ) -> Result<SourceArtifact, PromotionError> {
        let absolute = self.absolute(source.path());
        let current = fs::read_to_string(&absolute)
            .map_err(|error| PromotionError::Source(format!("{}: {error}", absolute.display())))?;
        if current != source.contents() {
            return Err(PromotionError::Conflict(source.path().clone()));
        }
        let kind = ArtifactKind::from_path(source.path().as_str())
            .ok_or_else(|| PromotionError::UnsupportedFormat(source.path().clone()))?;
        let patched = if kind == ArtifactKind::Gherkin {
            patch_gherkin(source.contents(), plan)?
        } else {
            patch_markdown(source.contents(), plan)?
        };
        atomic_replace(&absolute, &patched)?;
        Ok(SourceArtifact::new(source.path().clone(), patched))
    }
}

/// Supplies system UTC Unix-second timestamps for promotion.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Result<PromotionTimestamp, PromotionError> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| PromotionError::Clock(error.to_string()))?;
        let seconds = i64::try_from(duration.as_secs())
            .map_err(|error| PromotionError::Clock(error.to_string()))?;
        Ok(PromotionTimestamp::from_unix_seconds(seconds))
    }
}

#[allow(
    clippy::indexing_slicing,
    reason = "loop bounds are maintained against the same line vector while patching metadata"
)]
fn patch_markdown(contents: &str, plan: &PromotionPlan) -> Result<String, PromotionError> {
    let mut lines = split_lines(contents);
    if lines.first().map(|line| line.trim_end()) != Some("---") {
        return Err(PromotionError::UnsupportedFormat(plan.path().clone()));
    }
    let closing = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| (line.trim_end() == "---").then_some(index))
        .ok_or_else(|| PromotionError::UnsupportedFormat(plan.path().clone()))?;
    let mut status_found = false;
    let mut index = 1;
    while index < closing {
        let field = field_name(&lines[index]);
        if field == Some("status") {
            lines[index] =
                owned_line(&format!("status: {}", plan.target().as_str()), &lines[index]);
            status_found = true;
            index += 1;
            continue;
        }
        if is_promotion_field(field) {
            lines.remove(index);
            continue;
        }
        index += 1;
    }
    if !status_found {
        return Err(PromotionError::UnsupportedFormat(plan.path().clone()));
    }
    let insert_at = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(line_index, line)| (line.trim_end() == "---").then_some(line_index))
        .ok_or_else(|| PromotionError::UnsupportedFormat(plan.path().clone()))?;
    for line in promotion_lines(plan).into_iter().rev() {
        lines.insert(insert_at, line);
    }
    Ok(lines.concat())
}

#[allow(
    clippy::indexing_slicing,
    reason = "loop bounds are maintained against the same line vector while patching headers"
)]
fn patch_gherkin(contents: &str, plan: &PromotionPlan) -> Result<String, PromotionError> {
    let mut lines = split_lines(contents);
    let mut status_index = None;
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("Feature:") {
            break;
        }
        if trimmed.starts_with("# status:") {
            let prefix = &lines[index][..lines[index].len() - trimmed.len()];
            lines[index] =
                owned_line(&format!("{prefix}# status: {}", plan.target().as_str()), &lines[index]);
            status_index = Some(index);
            index += 1;
            continue;
        }
        if is_gherkin_promotion_header(trimmed) {
            lines.remove(index);
            continue;
        }
        index += 1;
    }
    let Some(insert_after) = status_index else {
        return Err(PromotionError::UnsupportedFormat(plan.path().clone()));
    };
    for line in gherkin_promotion_lines(plan).into_iter().rev() {
        lines.insert(insert_after + 1, line);
    }
    Ok(lines.concat())
}

fn atomic_replace(path: &Path, contents: &str) -> Result<(), PromotionError> {
    let parent =
        path.parent().ok_or_else(|| PromotionError::Write("target has no parent".to_string()))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| PromotionError::Write("target file name is not UTF-8".to_string()))?;
    let temp = parent.join(format!(".{file_name}.promotion.tmp"));
    {
        let mut file = fs::File::create(&temp)
            .map_err(|error| PromotionError::Write(format!("{}: {error}", temp.display())))?;
        file.write_all(contents.as_bytes())
            .map_err(|error| PromotionError::Write(format!("{}: {error}", temp.display())))?;
        file.sync_all()
            .map_err(|error| PromotionError::Write(format!("{}: {error}", temp.display())))?;
    }
    fs::rename(&temp, path)
        .map_err(|error| PromotionError::Write(format!("{}: {error}", path.display())))?;
    Ok(())
}

fn split_lines(contents: &str) -> Vec<String> {
    if contents.is_empty() {
        return Vec::new();
    }
    contents.split_inclusive('\n').map(str::to_string).collect()
}

fn owned_line(text: &str, original: &str) -> String {
    let newline = if original.ends_with('\n') { "\n" } else { "" };
    format!("{text}{newline}")
}

fn field_name(line: &str) -> Option<&str> {
    line.split_once(':').map(|(name, _)| name.trim())
}

fn is_promotion_field(field: Option<&str>) -> bool {
    matches!(field, Some("promoted_from" | "promoted_to" | "promoted_by" | "promoted_at"))
}

fn promotion_lines(plan: &PromotionPlan) -> Vec<String> {
    [
        format!("promoted_from: {}", plan.source().as_str()),
        format!("promoted_to: {}", plan.target().as_str()),
        format!("promoted_by: {}", plan.actor().as_str()),
        format!("promoted_at: {}", plan.promoted_at().as_unix_seconds()),
    ]
    .into_iter()
    .map(|line| format!("{line}\n"))
    .collect()
}

fn is_gherkin_promotion_header(trimmed: &str) -> bool {
    trimmed.starts_with("# promoted_from:")
        || trimmed.starts_with("# promoted_to:")
        || trimmed.starts_with("# promoted_by:")
        || trimmed.starts_with("# promoted_at:")
}

fn gherkin_promotion_lines(plan: &PromotionPlan) -> Vec<String> {
    [
        format!("# promoted_from: {}", plan.source().as_str()),
        format!("# promoted_to: {}", plan.target().as_str()),
        format!("# promoted_by: {}", plan.actor().as_str()),
        format!("# promoted_at: {}", plan.promoted_at().as_unix_seconds()),
    ]
    .into_iter()
    .map(|line| format!("{line}\n"))
    .collect()
}

#[cfg(test)]
#[allow(
    clippy::manual_let_else,
    reason = "tests keep fixture setup compact and abort on setup failure"
)]
mod tests {
    use application::ArtifactPromotionStore;
    use domain::{LifecycleState, PromotionActor};
    use tempfile::TempDir;

    use super::*;

    fn actor() -> PromotionActor {
        match PromotionActor::try_new("agent-1") {
            Ok(actor) => actor,
            Err(_) => std::process::abort(),
        }
    }

    fn path(value: &str) -> ArtifactPath {
        match ArtifactPath::try_new(value) {
            Ok(path) => path,
            Err(_) => std::process::abort(),
        }
    }

    /// Covers: REQ-006 FR-010, FR-012, and FR-014 — Markdown promotion patches only metadata.
    #[test]
    fn patches_markdown_frontmatter_and_preserves_body() {
        let temp = match TempDir::new() {
            Ok(temp) => temp,
            Err(_) => std::process::abort(),
        };
        let artifact_path = path("specs/feature/user-story.md");
        let absolute = temp.path().join(artifact_path.as_str());
        if fs::create_dir_all(absolute.parent().unwrap_or_else(|| temp.path())).is_err() {
            std::process::abort();
        }
        if fs::write(&absolute, "---\nstatus: draft\ntitle: T\n---\n# Body\n").is_err() {
            std::process::abort();
        }
        let mut store = FilesystemPromotionStore::new(temp.path());
        let source = match store.load(&artifact_path) {
            Ok(source) => source,
            Err(_) => std::process::abort(),
        };
        let plan = PromotionPlan::new(
            artifact_path.clone(),
            LifecycleState::Draft,
            LifecycleState::InReview,
            actor(),
            PromotionTimestamp::from_unix_seconds(7),
        );

        let result = store.commit(&source, &plan);

        assert!(result.is_ok());
        let contents = match fs::read_to_string(&absolute) {
            Ok(contents) => contents,
            Err(_) => std::process::abort(),
        };
        assert!(contents.contains("status: in-review\n"));
        assert!(contents.contains("promoted_from: draft\n"));
        assert!(contents.ends_with("# Body\n"));
    }

    /// Covers: REQ-006 FR-001 and FR-010 — Gherkin promotion patches status headers.
    #[test]
    fn patches_gherkin_headers() {
        let temp = match TempDir::new() {
            Ok(temp) => temp,
            Err(_) => std::process::abort(),
        };
        let artifact_path = path("specs/feature/scenarios.feature");
        let absolute = temp.path().join(artifact_path.as_str());
        if fs::create_dir_all(absolute.parent().unwrap_or_else(|| temp.path())).is_err() {
            std::process::abort();
        }
        if fs::write(&absolute, "# parent: US-001\n# status: draft\nFeature: Demo\n").is_err() {
            std::process::abort();
        }
        let mut store = FilesystemPromotionStore::new(temp.path());
        let source = match store.load(&artifact_path) {
            Ok(source) => source,
            Err(_) => std::process::abort(),
        };
        let plan = PromotionPlan::new(
            artifact_path.clone(),
            LifecycleState::Draft,
            LifecycleState::InReview,
            actor(),
            PromotionTimestamp::from_unix_seconds(7),
        );

        let result = store.commit(&source, &plan);

        assert!(result.is_ok());
        let contents = match fs::read_to_string(&absolute) {
            Ok(contents) => contents,
            Err(_) => std::process::abort(),
        };
        assert!(contents.contains("# status: in-review\n"));
        assert!(contents.contains("# promoted_to: in-review\n"));
        assert!(contents.ends_with("Feature: Demo\n"));
    }
}
