use std::{
    fs::{create_dir_all, read_to_string, write},
    path::Path,
    result::Result as StdResult,
};

use anyhow::{Context, Result};

pub trait Persistent {
    type State: serde::Serialize + serde::de::DeserializeOwned;

    fn write_state<P: AsRef<Path>>(path: P, state: &Self::State) -> Result<()> {
        let mut path = path.as_ref().to_path_buf();
        if path.extension().is_none() {
            create_dir_all(&path).context("Failed to create state directory")?;
            path = path.join("state.toml");
        }

        let state_string = toml::to_string_pretty(state).context("Failed to serialize state")?;
        write(path, state_string)?;

        Ok(())
    }

    fn read_state<P: AsRef<Path>>(path: P) -> Option<Self::State> {
        let path = path.as_ref();
        if !path.exists() {
            tracing::warn!("State file was not found");
            return None;
        }

        let Ok(state_json) = read_to_string(path) else {
            tracing::error!("Failed to read state file");
            return None;
        };

        let Ok(state): StdResult<Self::State, _> = toml::from_str(state_json.as_str()) else {
            tracing::error!("Failed to deserialize state");
            return None;
        };

        Some(state)
    }
}
