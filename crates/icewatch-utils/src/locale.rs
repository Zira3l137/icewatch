use anyhow::{Context, Result};
use serde::Deserialize;

use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Locale {
    pub language: String,
    pub country: String,
    pub strings: HashMap<String, HashMap<String, String>>,
}

impl Locale {
    pub fn get_string(&self, feature: &str, key: &str) -> &str {
        self.strings.get(feature).and_then(|s| s.get(key)).map(|s| s.as_str()).unwrap_or_else(
            || {
                tracing::warn!(
                    "Missing locale string [{}/{}] for {}-{}",
                    feature,
                    key,
                    self.language,
                    self.country
                );
                "Unknown"
            },
        )
    }

    pub fn as_tag(&self) -> String {
        format!("{}-{}", self.language, self.country)
    }
}

pub fn get_system_locale() -> String {
    sys_locale::get_locale().unwrap_or("en-US".to_owned())
}

pub fn read_available_locales<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Locale>> {
    Ok(fs::read_dir(path.as_ref())
        .context("Failed to read locales directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().ok().is_some_and(|ft| ft.is_dir()))
        .filter_map(|entry| {
            let Ok(locale_tag) = entry.file_name().into_string() else {
                return None;
            };

            Some((
                locale_tag,
                fs::read_dir(entry.path())
                    .ok()?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.file_type().ok().is_some_and(|ft| ft.is_file()))
                    .map(|entry| entry.path())
                    .collect::<Vec<_>>(),
            ))
        })
        .map(|(tag, paths)| {
            let parts: Vec<_> = tag.split("-").collect();
            let language = parts[0].to_owned();
            let country = parts[1].to_owned();
            let feature_strings: HashMap<String, HashMap<String, String>> = paths
                .iter()
                .filter_map(|path| {
                    let feature_name = path.file_stem()?.to_string_lossy().into_owned();
                    let contents = match fs::read_to_string(path) {
                        Ok(contents) => Some(contents),
                        Err(err) => {
                            tracing::error!("Failed to read locale file: {}", err);
                            None
                        }
                    }?;

                    let feature_string_layout: HashMap<String, String> =
                        match toml::from_str(&contents) {
                            Ok(layout) => Some(layout),
                            Err(err) => {
                                tracing::error!("Failed to parse locale file: {}", err);
                                None
                            }
                        }?;

                    Some((feature_name, feature_string_layout))
                })
                .collect();

            let locale = Locale { language, country, strings: feature_strings };
            (tag.clone(), locale)
        })
        .collect())
}
