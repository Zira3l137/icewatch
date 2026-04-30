use crate::{
    app::features::{main, settings},
    macros::{register_features, register_windows},
    rules::Rule,
};

use chrono::{DateTime, Local};
use iced::{
    Size, Theme,
    window::{Icon, Id, Settings as WindowSettings},
};
use icewatch_theme::load_available_themes;
use icewatch_utils::locale::Locale;
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

const THEMES_PATH: &str = "themes";

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub date: DateTime<Local>,
    pub startup_instant: Instant,
    pub last_redraw: Duration,
    pub icon: Option<Icon>,
    pub main_window_id: Option<Id>,
    pub windows: HashMap<Id, Window>,
    pub themes: HashMap<String, Theme>,
    pub locales: HashMap<String, Locale>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            icon: None,
            date: Local::now(),
            main_window_id: None,
            themes: HashMap::default(),
            windows: HashMap::default(),
            locales: HashMap::default(),
            startup_instant: Instant::now(),
            last_redraw: Duration::from_secs(0),
        }
    }
}

impl AppState {
    pub(crate) fn new(icon: Option<Icon>, locales: HashMap<String, Locale>) -> Self {
        Self {
            themes: load_available_themes(THEMES_PATH),
            date: Local::now(),
            icon,
            locales,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PersistentState {
    pub current_theme: String,
    pub current_locale: String,
    pub sorting_rules: Vec<Rule>,
    pub root_directory: PathBuf,
    pub sorting_enabled: bool,
    pub overwrite_existing: bool,
    pub watch_status: bool,
    pub purge_empty_directories: bool,
}

register_features!(main::Main, settings::Settings);

register_windows!(
    Main {
        settings: WindowSettings {
            size: Size::new(860.0, 600.0),
            exit_on_close_request: false,
            ..Default::default()
        },
        view_handler: main::view,
        input_handler: main::input,
        context: main::Context::new
    },
    Settings {
        settings: WindowSettings {
            size: Size::new(512.0, 256.0),
            exit_on_close_request: false,
            ..Default::default()
        },
        view_handler: settings::view,
        input_handler: settings::input,
        context: settings::Context::new
    }
);
