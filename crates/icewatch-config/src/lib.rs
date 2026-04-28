use std::{fs::read_to_string, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml::from_str;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub default_font: String,
}

pub fn read_settings<P: AsRef<Path>>(path: P) -> Result<Config> {
    let settings = read_to_string(path).context("Failed to read settings file")?;
    let settings: Config = from_str(&settings).context("Failed to parse settings")?;
    Ok(settings)
}
