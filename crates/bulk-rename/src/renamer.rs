use crate::config::{ApplyTo, TextFormatting};
use anyhow::{Context, Result};
use regex::Regex;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

// ── Data types ───────────────────────────────────────────────────────

/// Options that control how a rename preview is generated.
#[derive(Debug, Clone)]
pub struct RenameOptions {
    pub search: String,
    pub replace: String,
    pub use_regex: bool,
    pub match_all: bool,
    pub case_sensitive: bool,
    pub apply_to: ApplyTo,
    pub text_formatting: TextFormatting,
    pub enumerate: bool,
}

/// A single rename preview entry (before → after).
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

/// Options that control how entries are listed.
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub include_folders: bool,
    pub include_subfolders: bool,
}

// ── Renamer ──────────────────────────────────────────────────────────

pub struct Renamer {
    history: Vec<RenameOperation>,
}

impl Renamer {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    // ── Preview ──────────────────────────────────────────────────────

    /// Generate a preview of the rename operation without modifying anything.
    pub fn preview(&self, files: &[PathBuf], opts: &RenameOptions) -> Result<Vec<RenamePreview>> {
        // Validate regex early so we fail even on empty input.
        if opts.use_regex && !opts.search.is_empty() {
            let pattern = if opts.case_sensitive {
                opts.search.clone()
            } else {
                format!("(?i){}", opts.search)
            };
            Regex::new(&pattern).context("invalid regex pattern")?;
        }

        // When enumeration is active, protect ${...} tokens from the regex
        // engine by swapping them with a placeholder before replacement.
        let (effective_opts, enum_placeholder) = if opts.enumerate && opts.use_regex {
            let placeholder = "\x00MPT_ENUM\x00";
            let enum_re = Regex::new(r"\$\{[^}]*\}").unwrap();
            let safe_replace = enum_re.replace_all(&opts.replace, placeholder).into_owned();
            let mut eopts = opts.clone();
            eopts.replace = safe_replace;
            (eopts, Some(placeholder))
        } else {
            (opts.clone(), None)
        };

        let mut previews = Vec::with_capacity(files.len());
        let mut counter = EnumCounter::parse(&opts.replace);

        for file in files {
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // 1. Split into (target, rest) depending on ApplyTo.
            let (target, rest) = split_name(filename, opts.apply_to);

            // 2. Search & replace on the target part.
            let replaced = self.search_replace(&target, &effective_opts)?;

            // 3. Enumerate: expand counter patterns in the replacement.
            let replaced = if opts.enumerate {
                // Restore the placeholder back to original enum pattern, then expand.
                let with_patterns = if let Some(ph) = enum_placeholder {
                    restore_enum_placeholders(&replaced, ph, &opts.replace)
                } else {
                    replaced
                };
                counter.expand(&with_patterns)
            } else {
                replaced
            };

            // 4. Reassemble.
            let new_name = join_name(&replaced, &rest, opts.apply_to);

            // 5. Text formatting.
            let new_name = apply_formatting(&new_name, opts.text_formatting);

            let changed = new_name != filename;
            let renamed = file.with_file_name(&*new_name);

            previews.push(RenamePreview {
                original: file.clone(),
                renamed,
                changed,
            });
        }

        Ok(previews)
    }

    /// Backward-compatible helper that builds `RenameOptions` from bare
    /// regex pattern + replacement (match-all, case-sensitive, full name).
    pub fn preview_regex(
        &self,
        files: &[PathBuf],
        pattern: &str,
        replacement: &str,
    ) -> Result<Vec<RenamePreview>> {
        let opts = RenameOptions {
            search: pattern.into(),
            replace: replacement.into(),
            use_regex: true,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        self.preview(files, &opts)
    }

    // ── Execute / Undo ───────────────────────────────────────────────

    /// Execute the renames. Only renames entries that actually changed.
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

    // ── Listing ──────────────────────────────────────────────────────

    /// List entries in a directory with the given options.
    pub fn list_entries(dir: &Path, opts: &ListOptions) -> Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        Self::collect_entries(dir, opts, opts.include_subfolders, &mut entries)?;
        entries.sort();
        Ok(entries)
    }

    /// Convenience wrapper: list only files (legacy behaviour).
    pub fn list_files(dir: &Path) -> Result<Vec<PathBuf>> {
        Self::list_entries(dir, &ListOptions::default())
    }

    fn collect_entries(
        dir: &Path,
        opts: &ListOptions,
        recurse: bool,
        out: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let rd = fs::read_dir(dir)
            .with_context(|| format!("failed to read directory {}", dir.display()))?;

        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if opts.include_folders {
                    out.push(path.clone());
                }
                if recurse {
                    Self::collect_entries(&path, opts, true, out)?;
                }
            } else {
                out.push(path);
            }
        }

        Ok(())
    }

    // ── Internal helpers ─────────────────────────────────────────────

    /// Perform the search/replace on a single string slice.
    fn search_replace(&self, input: &str, opts: &RenameOptions) -> Result<String> {
        if opts.search.is_empty() {
            return Ok(input.to_string());
        }

        if opts.use_regex {
            self.search_replace_regex(input, opts)
        } else {
            Ok(self.search_replace_plain(input, opts))
        }
    }

    fn search_replace_regex(&self, input: &str, opts: &RenameOptions) -> Result<String> {
        let pattern = if opts.case_sensitive {
            opts.search.clone()
        } else {
            format!("(?i){}", opts.search)
        };
        let re = Regex::new(&pattern).context("invalid regex pattern")?;

        let result = if opts.match_all {
            re.replace_all(input, opts.replace.as_str())
        } else {
            re.replace(input, opts.replace.as_str())
        };

        Ok(result.into_owned())
    }

    fn search_replace_plain(&self, input: &str, opts: &RenameOptions) -> String {
        if opts.case_sensitive {
            if opts.match_all {
                input.replace(&opts.search, &opts.replace)
            } else {
                input.replacen(&opts.search, &opts.replace, 1)
            }
        } else {
            plain_replace_case_insensitive(input, &opts.search, &opts.replace, opts.match_all)
        }
    }
}

impl Default for Renamer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Free helpers ─────────────────────────────────────────────────────

/// Split a filename into (target, rest) depending on the `ApplyTo` mode.
///
/// - `FilenameOnly`         → target = stem,      rest = ".ext"
/// - `ExtensionOnly`        → target = ext,       rest = stem + "."
/// - `FilenameAndExtension` → target = full name,  rest = ""
fn split_name(filename: &str, apply_to: ApplyTo) -> (String, String) {
    match apply_to {
        ApplyTo::FilenameAndExtension => (filename.to_string(), String::new()),
        ApplyTo::FilenameOnly => {
            if let Some(dot) = filename.rfind('.') {
                (filename[..dot].to_string(), filename[dot..].to_string())
            } else {
                (filename.to_string(), String::new())
            }
        }
        ApplyTo::ExtensionOnly => {
            if let Some(dot) = filename.rfind('.') {
                (
                    filename[dot + 1..].to_string(),
                    filename[..=dot].to_string(),
                )
            } else {
                // No extension – nothing to operate on.
                (String::new(), filename.to_string())
            }
        }
    }
}

/// Reassemble the filename from the modified part and the untouched part.
fn join_name(modified: &str, rest: &str, apply_to: ApplyTo) -> String {
    match apply_to {
        ApplyTo::FilenameAndExtension => modified.to_string(),
        ApplyTo::FilenameOnly => format!("{modified}{rest}"),
        ApplyTo::ExtensionOnly => format!("{rest}{modified}"),
    }
}

/// Apply text formatting to the whole filename (including extension).
fn apply_formatting(name: &str, fmt: TextFormatting) -> Cow<'_, str> {
    match fmt {
        TextFormatting::None => Cow::Borrowed(name),
        TextFormatting::Lowercase => Cow::Owned(name.to_lowercase()),
        TextFormatting::Uppercase => Cow::Owned(name.to_uppercase()),
        TextFormatting::TitleCase => {
            let mut chars = name.chars();
            let result = match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            };
            Cow::Owned(result)
        }
        TextFormatting::CapitalizeEachWord => {
            let mut result = String::with_capacity(name.len());
            let mut capitalize_next = true;
            for c in name.chars() {
                if c == ' ' || c == '_' || c == '-' || c == '.' {
                    capitalize_next = true;
                    result.push(c);
                } else if capitalize_next {
                    for u in c.to_uppercase() {
                        result.push(u);
                    }
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
            }
            Cow::Owned(result)
        }
    }
}

/// Case-insensitive plain-text replacement.
fn plain_replace_case_insensitive(
    input: &str,
    search: &str,
    replace: &str,
    match_all: bool,
) -> String {
    if search.is_empty() {
        return input.to_string();
    }

    let lower_input = input.to_lowercase();
    let lower_search = search.to_lowercase();
    let mut result = String::with_capacity(input.len());
    let mut start = 0;

    while let Some(pos) = lower_input[start..].find(&lower_search) {
        let abs_pos = start + pos;
        result.push_str(&input[start..abs_pos]);
        result.push_str(replace);
        start = abs_pos + search.len();

        if !match_all {
            break;
        }
    }

    result.push_str(&input[start..]);
    result
}

/// Restore enum placeholders back to the original `${...}` patterns so
/// `EnumCounter::expand` can process them.  Each placeholder occurrence
/// is replaced with the corresponding `${...}` token from `original`.
fn restore_enum_placeholders(text: &str, placeholder: &str, original: &str) -> String {
    let enum_re = Regex::new(r"\$\{[^}]*\}").unwrap();
    let tokens: Vec<&str> = enum_re.find_iter(original).map(|m| m.as_str()).collect();
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;
    let mut idx = 0;

    while let Some(pos) = remaining.find(placeholder) {
        result.push_str(&remaining[..pos]);
        let token = tokens.get(idx).copied().unwrap_or("${}");
        result.push_str(token);
        remaining = &remaining[pos + placeholder.len()..];
        idx += 1;
    }
    result.push_str(remaining);
    result
}

// ── Enumeration counter ──────────────────────────────────────────────

/// Handles `${}` enumeration patterns in replacement strings.
///
/// Supported patterns (matching PowerRename):
///   `${}`                          – simple 0-based counter
///   `${start=N}`                   – counter starting at N
///   `${increment=N}`               – step by N
///   `${padding=N}`                 – zero-pad to N digits
///   `${start=N;increment=M;padding=P}` – combined
struct EnumCounter {
    current: i64,
    increment: i64,
    padding: usize,
    has_pattern: bool,
}

impl EnumCounter {
    /// Parse the replacement string looking for `${...}` enum patterns.
    fn parse(replacement: &str) -> Self {
        let mut start: i64 = 0;
        let mut increment: i64 = 1;
        let mut padding: usize = 0;
        let mut has_pattern = false;

        // Look for ${...} pattern.
        let re = Regex::new(r"\$\{([^}]*)\}").unwrap();
        if re.is_match(replacement) {
            has_pattern = true;
            // Parse the first match to extract options.
            if let Some(caps) = re.captures(replacement) {
                let inner = &caps[1];
                for part in inner.split(';') {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }
                    if let Some(v) = part.strip_prefix("start=") {
                        start = v.trim().parse().unwrap_or(0);
                    } else if let Some(v) = part.strip_prefix("increment=") {
                        increment = v.trim().parse().unwrap_or(1);
                    } else if let Some(v) = part.strip_prefix("padding=") {
                        padding = v.trim().parse().unwrap_or(0);
                    }
                }
            }
        }

        Self {
            current: start,
            increment,
            padding,
            has_pattern,
        }
    }

    /// Expand all `${...}` patterns in the template with the current counter
    /// value, then advance the counter.
    fn expand(&mut self, input: &str) -> String {
        if !self.has_pattern {
            return input.to_string();
        }

        let formatted = if self.padding > 0 {
            format!("{:0>width$}", self.current, width = self.padding)
        } else {
            self.current.to_string()
        };

        // Replace all ${...} patterns in the input.
        let re = Regex::new(r"\$\{[^}]*\}").unwrap();
        let result = re.replace_all(input, formatted.as_str()).into_owned();

        self.current += self.increment;
        result
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- Legacy API ---------------------------------------------------

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
        assert!(!previews[2].changed);
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

    // -- Plain text search --------------------------------------------

    #[test]
    fn plain_text_replace_all() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/foo-bar-foo.txt")];
        let opts = RenameOptions {
            search: "foo".into(),
            replace: "baz".into(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/baz-bar-baz.txt"));
    }

    #[test]
    fn plain_text_replace_first() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/foo-bar-foo.txt")];
        let opts = RenameOptions {
            search: "foo".into(),
            replace: "baz".into(),
            use_regex: false,
            match_all: false,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/baz-bar-foo.txt"));
    }

    // -- Case insensitive ---------------------------------------------

    #[test]
    fn case_insensitive_plain() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/FooBar.txt")];
        let opts = RenameOptions {
            search: "foobar".into(),
            replace: "replaced".into(),
            use_regex: false,
            match_all: true,
            case_sensitive: false,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/replaced.txt"));
    }

    #[test]
    fn case_insensitive_regex() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/FooBar.txt")];
        let opts = RenameOptions {
            search: "foobar".into(),
            replace: "replaced".into(),
            use_regex: true,
            match_all: true,
            case_sensitive: false,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/replaced.txt"));
    }

    // -- Apply to -----------------------------------------------------

    #[test]
    fn apply_to_filename_only() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/foo.foo")];
        let opts = RenameOptions {
            search: "foo".into(),
            replace: "bar".into(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameOnly,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        // Only stem changes, extension stays .foo
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/bar.foo"));
    }

    #[test]
    fn apply_to_extension_only() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/foo.foo")];
        let opts = RenameOptions {
            search: "foo".into(),
            replace: "bar".into(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::ExtensionOnly,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        // Only extension changes, stem stays foo
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/foo.bar"));
    }

    // -- Text formatting ----------------------------------------------

    #[test]
    fn formatting_lowercase() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/HELLO_WORLD.TXT")];
        let opts = RenameOptions {
            search: String::new(),
            replace: String::new(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::Lowercase,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/hello_world.txt"));
    }

    #[test]
    fn formatting_uppercase() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/hello.txt")];
        let opts = RenameOptions {
            search: String::new(),
            replace: String::new(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::Uppercase,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/HELLO.TXT"));
    }

    #[test]
    fn formatting_title_case() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/hello world.txt")];
        let opts = RenameOptions {
            search: String::new(),
            replace: String::new(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::TitleCase,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/Hello world.txt"));
    }

    #[test]
    fn formatting_capitalize_each_word() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/hello world-foo.txt")];
        let opts = RenameOptions {
            search: String::new(),
            replace: String::new(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::CapitalizeEachWord,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(
            previews[0].renamed,
            PathBuf::from("/tmp/Hello World-Foo.Txt")
        );
    }

    // -- Enumeration --------------------------------------------------

    #[test]
    fn enumerate_simple() {
        let renamer = Renamer::new();
        let files = vec![
            PathBuf::from("/tmp/a.txt"),
            PathBuf::from("/tmp/b.txt"),
            PathBuf::from("/tmp/c.txt"),
        ];
        let opts = RenameOptions {
            search: r"(.+)\.txt".into(),
            replace: "${}_$1.txt".into(),
            use_regex: true,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: true,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/0_a.txt"));
        assert_eq!(previews[1].renamed, PathBuf::from("/tmp/1_b.txt"));
        assert_eq!(previews[2].renamed, PathBuf::from("/tmp/2_c.txt"));
    }

    #[test]
    fn enumerate_with_options() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/a.jpg"), PathBuf::from("/tmp/b.jpg")];
        let opts = RenameOptions {
            search: r"(.+)\.jpg".into(),
            replace: "img_${start=10;increment=5;padding=4}.jpg".into(),
            use_regex: true,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: true,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert_eq!(previews[0].renamed, PathBuf::from("/tmp/img_0010.jpg"));
        assert_eq!(previews[1].renamed, PathBuf::from("/tmp/img_0015.jpg"));
    }

    // -- Edge cases ---------------------------------------------------

    #[test]
    fn empty_search_returns_unchanged() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/test.txt")];
        let opts = RenameOptions {
            search: String::new(),
            replace: "anything".into(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::FilenameAndExtension,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        assert!(!previews[0].changed);
    }

    #[test]
    fn extension_only_no_extension() {
        let renamer = Renamer::new();
        let files = vec![PathBuf::from("/tmp/Makefile")];
        let opts = RenameOptions {
            search: "Make".into(),
            replace: "Cake".into(),
            use_regex: false,
            match_all: true,
            case_sensitive: true,
            apply_to: ApplyTo::ExtensionOnly,
            text_formatting: TextFormatting::None,
            enumerate: false,
        };
        let previews = renamer.preview(&files, &opts).unwrap();
        // No extension → nothing to operate on → unchanged.
        assert!(!previews[0].changed);
    }
}
