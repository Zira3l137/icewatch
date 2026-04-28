mod macros;

use anyhow::{Context, Result};
use iced::{
    Color, Theme,
    theme::{Base, Palette},
};
use macros::register_themes;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct UserTheme {
    pub name: String,
    pub background: Option<String>,
    pub text: Option<String>,
    pub primary: Option<String>,
    pub success: Option<String>,
    pub danger: Option<String>,
    pub warning: Option<String>,
}

impl From<UserTheme> for Theme {
    fn from(value: UserTheme) -> Self {
        let palette = Palette {
            background: hex_to_color(&value.background.unwrap_or("#FFFFFF".to_owned())),
            text: hex_to_color(&value.text.unwrap_or("#000000".to_owned())),
            primary: hex_to_color(&value.primary.unwrap_or("#0000FF".to_owned())),
            success: hex_to_color(&value.success.unwrap_or("#00FF00".to_owned())),
            warning: hex_to_color(&value.warning.unwrap_or("#FFFF00".to_owned())),
            danger: hex_to_color(&value.danger.unwrap_or("#FF0000".to_owned())),
        };
        Theme::custom(value.name, palette)
    }
}

pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

register_themes!(
    RosePine => Palette {
        background: Color::from_rgb(0.1, 0.09, 0.14),
        text: Color::from_rgb(0.88, 0.87, 0.96),
        success: Color::from_rgb(0.61, 0.81, 0.85),
        warning: Color::from_rgb(0.96, 0.76, 0.47),
        primary: Color::from_rgb(0.77, 0.65, 0.91),
        danger: Color::from_rgb(0.92, 0.44, 0.57),
    }
);

pub const fn default_themes() -> [Theme; 22] {
    [
        Theme::Light,
        Theme::Dark,
        Theme::Dracula,
        Theme::Nord,
        Theme::SolarizedLight,
        Theme::SolarizedDark,
        Theme::GruvboxLight,
        Theme::GruvboxDark,
        Theme::CatppuccinLatte,
        Theme::CatppuccinFrappe,
        Theme::CatppuccinMacchiato,
        Theme::CatppuccinMocha,
        Theme::TokyoNight,
        Theme::TokyoNightStorm,
        Theme::TokyoNightLight,
        Theme::KanagawaWave,
        Theme::KanagawaDragon,
        Theme::KanagawaLotus,
        Theme::Moonfly,
        Theme::Nightfly,
        Theme::Oxocarbon,
        Theme::Ferra,
    ]
}

pub fn read_user_themes<P: AsRef<Path>>(path: P) -> Result<Vec<iced::Theme>> {
    let path = path.as_ref();
    Ok(path
        .read_dir()
        .context("Failed to read user themes directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().is_ok_and(|ft| ft.is_file())
                && entry.file_name().to_string_lossy().to_lowercase().ends_with(".toml")
            {
                let path = entry.path();
                let content = fs::read_to_string(&path).ok()?;
                let theme: UserTheme = toml::from_str(&content).ok()?;
                Some(theme.into())
            } else {
                None
            }
        })
        .collect())
}

pub fn load_available_themes<P: AsRef<Path>>(path: P) -> HashMap<String, iced::Theme> {
    tracing::info!("Loading registered themes");
    let registered_themes = registered_themes();

    tracing::info!("Loading user themes");
    let user_themes = read_user_themes(path)
        .inspect_err(|e| tracing::error!("Failed to read user themes: {}", e))
        .unwrap_or_default();

    registered_themes
        .iter()
        .chain(user_themes.iter())
        .chain(default_themes().iter())
        .map(|t| (t.name().to_owned(), t.clone()))
        .collect()
}
