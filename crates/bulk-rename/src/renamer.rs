use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// A single rename operation (before -> after).
#[derive(Debug, Clone)]
pub struct RenamePreview {
    pub original: PathBuf,
    pub renamed: PathBuf,
    pub changed: bool,
}

/// A completed rename operation, stored for undo.
#[derive(Debug, Clone)]
pub struct RenameOperation {
    pub renames: Vec<(PathBuf, PathBuf)>,
}

pub struct Renamer {
    history: Vec<RenameOperation>,
}

impl Renamer {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Preview what a regex search/replace would do to a list of files.
    pub fn preview_regex(
        &self,
        files: &[PathBuf],
        pattern: &str,
        replacement: &str,
    ) -> Result<Vec<RenamePreview>> {
        let re = Regex::new(pattern).context("invalid regex pattern")?;
        let mut previews = Vec::new();

        for file in files {
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let new_name = re.replace_all(filename, replacement);
            let changed = new_name.as_ref() != filename;
            let renamed = file.with_file_name(new_name.as_ref());

            previews.push(RenamePreview {
                original: file.clone(),
                renamed,
                changed,
            });
        }

        Ok(previews)
    }

    /// Execute the renames. Only renames files that actually changed.
    pub fn execute(&mut self, previews: &[RenamePreview]) -> Result<RenameOperation> {
        let mut renames = Vec::new();

        for preview in previews {
            if !preview.changed {
                continue;
            }

            if preview.renamed.exists() {
                anyhow::bail!("target already exists: {}", preview.renamed.display());
            }

            fs::rename(&preview.original, &preview.renamed).with_context(|| {
                format!(
                    "failed to rename {} -> {}",
                    preview.original.display(),
                    preview.renamed.display()
                )
            })?;

            info!(
                "Renamed: {} -> {}",
                preview.original.display(),
                preview.renamed.display()
            );
            renames.push((preview.original.clone(), preview.renamed.clone()));
        }

        let op = RenameOperation { renames };
        self.history.push(op.clone());
        Ok(op)
    }

    /// Undo the last rename operation.
    pub fn undo(&mut self) -> Result<()> {
        let op = self
            .history
            .pop()
            .ok_or_else(|| anyhow::anyhow!("nothing to undo"))?;

        for (original, renamed) in op.renames.iter().rev() {
            fs::rename(renamed, original).with_context(|| {
                format!(
                    "failed to undo rename {} -> {}",
                    renamed.display(),
                    original.display()
                )
            })?;
            info!("Undone: {} -> {}", renamed.display(), original.display());
        }

        Ok(())
    }

    /// List files in a directory.
    pub fn list_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files: Vec<PathBuf> = fs::read_dir(dir)
            .with_context(|| format!("failed to read directory {}", dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        files.sort();
        Ok(files)
    }
}

impl Default for Renamer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_regex_replace() {
        let renamer = Renamer::new();
        let files = vec![
            PathBuf::from("/tmp/photo_001.jpg"),
            PathBuf::from("/tmp/photo_002.jpg"),
            PathBuf::from("/tmp/document.txt"),
        ];

        let previews = renamer
            .preview_regex(&files, r"photo_(\d+)", "img_$1")
            .unwrap();

        assert!(previews[0].changed);
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/img_001.jpg"));
        assert!(previews[1].changed);
        assert_eq!(previews[1].renamed, PathBuf::from("/tmp/img_002.jpg"));
        assert!(!previews[2].changed); // document.txt unchanged
    }

    #[test]
    fn preview_no_match() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/test.txt")];
        let previews = renamer.preview_regex(&files, "xyz", "abc").unwrap();
        assert!(!previews[0].changed);
    }

    #[test]
    fn invalid_regex_errors() {
        let renamer = Renamer::new();
        let result = renamer.preview_regex(&[], "[invalid", "");
        assert!(result.is_err());
    }
}
