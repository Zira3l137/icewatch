use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Rule {
    pub criterion: CriterionKind,
    pub destination: PathBuf,
}

impl Rule {
    pub fn new(criterion: impl Into<CriterionKind>, destination: impl AsRef<Path>) -> Result<Self> {
        let destination = destination.as_ref();
        if destination.as_os_str().is_empty() {
            return Err(anyhow!("destination cannot be empty"));
        }
        Ok(Self { criterion: criterion.into(), destination: destination.to_path_buf() })
    }

    pub fn applies_to(&self, path: &Path) -> bool {
        self.criterion.matches(path)
    }
}

// Wire the criterion types through CriterionKind

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CriterionKind {
    ByExtension(ByExtension),
    ByName(ByName),
}

impl CriterionKind {
    fn matches(&self, path: &Path) -> bool {
        match self {
            Self::ByExtension(c) => c.matches(path),
            Self::ByName(c) => c.matches(path),
        }
    }
}

impl From<ByExtension> for CriterionKind {
    fn from(c: ByExtension) -> Self {
        Self::ByExtension(c)
    }
}

impl From<ByName> for CriterionKind {
    fn from(c: ByName) -> Self {
        Self::ByName(c)
    }
}

impl std::fmt::Display for CriterionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByExtension(c) => write!(f, "{}", c),
            Self::ByName(c) => write!(f, "{}", c),
        }
    }
}

// Introduce a separate type for each criterion

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ByExtension {
    pub extensions: Vec<String>,
}

impl ByExtension {
    const SEPARATORS: &'static [char] = &[',', ';'];

    pub fn new(extensions: impl Into<String>) -> Self {
        let haystack = extensions.into();
        let mut extensions = Vec::new();
        for sep in Self::SEPARATORS {
            if haystack.contains(*sep) {
                extensions.extend(haystack.split(*sep).map(|e| e.trim_ascii().to_owned()));
            }
        }
        if extensions.is_empty() {
            extensions.push(haystack.trim_ascii().to_owned());
        }

        Self { extensions }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ByName {
    pub starts_with: Option<String>,
    pub ends_with: Option<String>,
    pub contains: Option<String>,
}

impl ByName {
    #[expect(dead_code)]
    pub(crate) fn new(
        starts_with: Option<impl Into<String>>,
        ends_with: Option<impl Into<String>>,
        contains: Option<impl Into<String>>,
    ) -> Self {
        Self {
            starts_with: starts_with.map(|s| s.into()),
            ends_with: ends_with.map(|s| s.into()),
            contains: contains.map(|s| s.into()),
        }
    }

    #[expect(dead_code)]
    pub(crate) fn by_prefix(prefix: impl Into<String>) -> Self {
        Self { starts_with: Some(prefix.into()), ends_with: None, contains: None }
    }

    #[expect(dead_code)]
    pub(crate) fn by_suffix(suffix: impl Into<String>) -> Self {
        Self { starts_with: None, ends_with: Some(suffix.into()), contains: None }
    }

    #[expect(dead_code)]
    pub(crate) fn by_contains(contains: impl Into<String>) -> Self {
        Self { starts_with: None, ends_with: None, contains: Some(contains.into()) }
    }

    #[expect(dead_code)]
    pub(crate) fn containing(name: impl Into<String>) -> Self {
        Self { starts_with: None, ends_with: None, contains: Some(name.into()) }
    }
}

impl std::fmt::Display for ByName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "By name: prefix={:?}, suffix={:?}, contains={:?}",
            self.starts_with, self.ends_with, self.contains
        )
    }
}

impl std::fmt::Display for ByExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "By extensions: {}", self.extensions.join(", "))
    }
}

// Implement Criterion trait for each criterion type

trait Criterion {
    fn matches(&self, path: &Path) -> bool;
}

impl Criterion for ByExtension {
    fn matches(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| self.extensions.iter().any(|ext| ext.eq_ignore_ascii_case(e)))
            .unwrap_or(false)
    }
}

impl Criterion for ByName {
    fn matches(&self, path: &Path) -> bool {
        let Some(name) = path.file_stem().and_then(|n| n.to_str()) else {
            return false;
        };
        self.starts_with.as_deref().is_none_or(|s| name.starts_with(s))
            && self.ends_with.as_deref().is_none_or(|s| name.ends_with(s))
            && self.contains.as_deref().is_none_or(|s| name.contains(s))
    }
}
