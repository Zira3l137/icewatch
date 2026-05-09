use std::path::PathBuf;

pub(crate) const DOWNLOAD_TEMP_EXTENSIONS: &[&str] = &[
    "part",       // Firefox
    "crdownload", // Chrome
    "download",   // Safari / misc
    "opdownload", // Opera
    "tmp",
];

/// A helper enum used by a sorting criterion selector in the Ui.
#[derive(Debug, Clone, Default)]
pub(crate) enum Criterion {
    /// Sorting files by prefix, suffix and/or sub-string in their name.
    ByName,

    /// Sorting files by their extension.
    #[default]
    ByExtension,
}

impl std::fmt::Display for Criterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Criterion::ByName => write!(f, "By name"),
            Criterion::ByExtension => write!(f, "By extension"),
        }
    }
}

/// Represents the current IO pipeline stage.
#[derive(Debug, Clone, Default)]
pub(crate) enum PipelineStage {
    /// Represents the full indexing stage, scanning the entire root directory recursively.
    #[default]
    IndexFull,

    /// Represents a partial indexing stage, scanning only the specified paths.
    IndexPaths(Vec<(PathBuf, bool)>),

    /// Represents a stage that purges empty directories recursively at the root.
    PurgeEmptyDirs,

    /// Represents a stage that sorts the indexed files according to existing sorting rules and
    /// replaces them in the filesystem to represent the sorted order.
    Sort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JournalEntrySection {
    Today,
    Yesterday,
    All,
}

#[derive(Debug, Clone)]
pub(crate) enum WatcherEvent {
    Created(Vec<(PathBuf, bool)>),
    Removed(Vec<PathBuf>),
    Renamed(Vec<(PathBuf, PathBuf)>),
}
