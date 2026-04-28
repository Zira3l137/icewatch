#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;
mod macros;
mod rules;

use std::{path::Path, sync::LazyLock};

use icewatch_config::read_settings;
use icewatch_utils::{cli, io::read_fonts, locale::read_available_locales, logging};

use anyhow::{Context, Result, anyhow};
use iced::{Font, Settings, daemon, window::icon};
use logging::setup_logger;

use crate::app::App;

static CONFIG: LazyLock<&Path> = LazyLock::new(|| Path::new("app_config.toml"));
static LOCALES: LazyLock<&Path> = LazyLock::new(|| Path::new("resources/locales"));
static IMAGES: LazyLock<&Path> = LazyLock::new(|| Path::new("resources/images"));
static FONTS: LazyLock<&Path> = LazyLock::new(|| Path::new("resources/fonts"));

fn main() -> Result<()> {
    let args = cli::parse();
    let default_log_file = format!("{}.log", env!("WORKSPACE_NAME"));
    let log_file = args.log_to_file.then_some(default_log_file).or(None);
    setup_logger(args.verbosity, log_file).context("Failed to initialize logger.")?;

    let config = read_settings(*CONFIG).context("Failed to read application settings.")?;
    let fonts = read_fonts(*FONTS).context("Failed to read application fonts.")?;
    let locales = read_available_locales(*LOCALES).context("Failed to load available locales")?;

    if locales.is_empty() {
        tracing::error!("No locales found");
        return Err(anyhow!("No locales found"));
    }

    let icon_path = IMAGES.join("icon.ico");
    let icon = icon::from_file(&icon_path)
        .inspect_err(|e| {
            tracing::error!(
                "Failed to load application icon \"{}\": {e}",
                &icon_path.to_str().unwrap_or_default()
            )
        })
        .ok();

    let default_font_name = config.default_font;
    let default_font = Font::with_name(Box::leak(default_font_name.into_boxed_str()));
    let settings = Settings { default_font, fonts, ..Default::default() };

    daemon(move || App::new(icon.as_ref(), &locales), App::update, App::view)
        .subscription(App::subscription)
        .style(App::style)
        .theme(App::theme)
        .title(App::title)
        .settings(settings)
        .run()
        .context("Failed to initialize application daemon.")
}
