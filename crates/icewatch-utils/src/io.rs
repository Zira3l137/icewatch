use std::{borrow::Cow, path::Path};

use anyhow::{Context, Result};

pub fn read_fonts<P: AsRef<Path>>(path: P) -> Result<Vec<Cow<'static, [u8]>>> {
    let path = path.as_ref();
    let loaded_fonts = path
        .read_dir()
        .context("Failed to read directory")?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension().map(|e| e.eq_ignore_ascii_case("ttf")).unwrap_or(false) {
                let bytes = std::fs::read(path).ok()?;
                return Some(Cow::Owned(bytes));
            }
            None
        })
        .collect();
    Ok(loaded_fonts)
}
