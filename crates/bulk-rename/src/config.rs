use serde::{Deserialize, Serialize};

/// How to apply the search/replace operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ApplyTo {
    /// Only modify the filename (stem), keep extension untouched.
    FilenameOnly,
    /// Only modify the extension, keep filename untouched.
    ExtensionOnly,
    /// Modify both filename and extension together.
    #[default]
    FilenameAndExtension,
}

/// Post-processing text formatting to apply after replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TextFormatting {
    #[default]
    None,
    /// all lowercase
    Lowercase,
    /// ALL UPPERCASE
    Uppercase,
    /// Title case (first character of the name capitalised)
    TitleCase,
    /// Capitalize Each Word
    CapitalizeEachWord,
}

fn default_true() -> bool {
    true
}

/// Persisted configuration for the Bulk Rename module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRenameConfig {
    /// Interpret the search field as a regular expression.
    #[serde(default)]
    pub use_regex: bool,

    /// Replace every occurrence (true) or only the first one (false).
    #[serde(default = "default_true")]
    pub match_all_occurrences: bool,

    /// Case-sensitive matching.
    #[serde(default)]
    pub case_sensitive: bool,

    /// Which part of the filename the operation targets.
    #[serde(default)]
    pub apply_to: ApplyTo,

    /// Include folders (not just files) in the listing.
    #[serde(default)]
    pub include_folders: bool,

    /// Recurse into subfolders when listing entries.
    #[serde(default)]
    pub include_subfolders: bool,

    /// Post-replacement text formatting.
    #[serde(default)]
    pub text_formatting: TextFormatting,

    /// Enable enumeration patterns (`${}`) in the replacement string.
    #[serde(default)]
    pub enumerate_items: bool,
}

impl Default for BulkRenameConfig {
    fn default() -> Self {
        Self {
            use_regex: false,
            match_all_occurrences: true,
            case_sensitive: false,
            apply_to: ApplyTo::default(),
            include_folders: false,
            include_subfolders: false,
            text_formatting: TextFormatting::default(),
            enumerate_items: false,
        }
    }
}
