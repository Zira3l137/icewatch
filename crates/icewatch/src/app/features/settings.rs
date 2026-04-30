use std::collections::HashMap;

use iced::Element;
use iced::Length;
use iced::Task;
use iced::Theme;
use iced::keyboard;
use iced::mouse;
use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::toggler;
use iced::window::Id;
use icewatch_utils::locale::Locale;

use super::CONTAINER_PADDING;
use super::DEFAULT_THEME;
use super::ICON_SIZE;
use super::ROW_PADDING;
use super::ROW_SPACING;
use crate::app::App;
use crate::app::message::InputEvent;
use crate::app::message::Message as GlobalMessage;
use crate::app::state::FeatureMessage;

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    theme_state: combo_box::State<String>,
    locale_state: combo_box::State<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Context<'a> {
    feature_state: &'a State,
    current_theme: &'a str,
    current_locale: &'a str,
    sorting_enabled: &'a bool,
    overwrite_existing: &'a bool,
    purge_empty: &'a bool,
    themes: &'a HashMap<String, Theme>,
    locales: &'a HashMap<String, Locale>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(app: &'a App) -> Self {
        Self {
            feature_state: &app.features_state.settings,
            current_theme: &app.persistent_state.current_theme,
            current_locale: &app.persistent_state.current_locale,
            sorting_enabled: &app.persistent_state.sorting_enabled,
            purge_empty: &app.persistent_state.purge_empty_directories,
            overwrite_existing: &app.persistent_state.overwrite_existing,
            themes: &app.app_state.themes,
            locales: &app.app_state.locales,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ContextMut<'a> {
    feature_state: &'a mut State,
    current_theme: &'a mut String,
    current_locale: &'a mut String,
    sorting_enabled: &'a mut bool,
    overwrite_existing: &'a mut bool,
    purge_empty: &'a mut bool,
    themes: &'a mut HashMap<String, Theme>,
    locales: &'a mut HashMap<String, Locale>,
}

impl<'a> ContextMut<'a> {
    pub(crate) fn new(app: &'a mut App) -> Self {
        Self {
            feature_state: &mut app.features_state.settings,
            current_theme: &mut app.persistent_state.current_theme,
            current_locale: &mut app.persistent_state.current_locale,
            sorting_enabled: &mut app.persistent_state.sorting_enabled,
            purge_empty: &mut app.persistent_state.purge_empty_directories,
            overwrite_existing: &mut app.persistent_state.overwrite_existing,
            themes: &mut app.app_state.themes,
            locales: &mut app.app_state.locales,
        }
    }
}

pub(crate) fn init(ctx: ContextMut<'_>) {
    let themes = ctx.themes.keys().cloned().collect();
    let locales = ctx.locales.keys().cloned().collect();
    ctx.feature_state.theme_state = combo_box::State::new(themes);
    ctx.feature_state.locale_state = combo_box::State::new(locales);
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    ThemeSwitch(String),
    LocaleSwitch(String),
    SortingToggle(bool),
    OverwriteToggle(bool),
    PurgeEmptyToggle(bool),
}

impl From<Message> for GlobalMessage {
    fn from(msg: Message) -> GlobalMessage {
        GlobalMessage::Feature(FeatureMessage::Settings(msg))
    }
}

pub(crate) fn update<'a>(msg: Message, ctx: ContextMut<'a>) -> Task<GlobalMessage> {
    match msg {
        Message::ThemeSwitch(theme_name) => {
            *ctx.current_theme = theme_name;
            Task::none()
        }
        Message::LocaleSwitch(locale_tag) => {
            *ctx.current_locale = locale_tag;
            Task::none()
        }
        Message::PurgeEmptyToggle(new_state) => {
            *ctx.purge_empty = new_state;
            Task::none()
        }
        Message::OverwriteToggle(new_state) => {
            *ctx.overwrite_existing = new_state;
            Task::none()
        }
        Message::SortingToggle(new_state) => {
            *ctx.sorting_enabled = new_state;
            Task::none()
        }
    }
}

pub(crate) fn view<'a>(ctx: Context<'a>, _window_id: Id) -> Element<'a, GlobalMessage> {
    let current_locale = ctx.current_locale;
    let current_theme = ctx.current_theme;

    let theme = ctx
        .themes
        .get(current_theme)
        .unwrap_or_else(|| ctx.themes.get(DEFAULT_THEME).unwrap_or(&iced::Theme::Dark));
    let palette = theme.extended_palette();

    let locale = ctx.locales.get(current_locale).expect("locale not found");
    let local = |key: &str| locale.get_string("settings", key);

    let locale_icon = text(local("locale_icon"))
        .color(palette.primary.base.color)
        .size(ICON_SIZE)
        .center()
        .height(Length::Fill);

    let locale_switcher =
        combo_box(&ctx.feature_state.locale_state, ctx.current_locale, None, |l| {
            Message::LocaleSwitch(l.to_owned()).into()
        });
    let locale_row = row([locale_icon.into(), locale_switcher.into()]);

    let theme_icon = text(local("theme_icon"))
        .color(palette.primary.base.color)
        .size(ICON_SIZE)
        .center()
        .height(Length::Fill);

    let theme_switcher = combo_box(&ctx.feature_state.theme_state, ctx.current_theme, None, |t| {
        Message::ThemeSwitch(t.to_owned()).into()
    });
    let theme_row = row([theme_icon.into(), theme_switcher.into()]);

    let sorting_toggle: Element<'a, GlobalMessage> = toggler(*ctx.sorting_enabled)
        .label(local("sorting_toggle"))
        .on_toggle(|b| Message::SortingToggle(b).into())
        .into();

    let overwrite_toggle: Element<'a, GlobalMessage> = toggler(*ctx.overwrite_existing)
        .label(local("overwrite_toggle"))
        .on_toggle(|b| Message::OverwriteToggle(b).into())
        .into();

    let purge_empty_toggle: Element<'a, GlobalMessage> = toggler(*ctx.purge_empty)
        .label(local("purge_empty_toggle"))
        .on_toggle(|b| Message::PurgeEmptyToggle(b).into())
        .into();

    container(
        column([
            theme_row.into(),
            locale_row.into(),
            sorting_toggle,
            overwrite_toggle,
            purge_empty_toggle,
        ])
        .padding(ROW_PADDING)
        .spacing(ROW_SPACING),
    )
    .align_top(Length::Shrink)
    .padding(CONTAINER_PADDING)
    .style(container::bordered_box)
    .into()
}

pub(crate) fn input(input: &InputEvent) -> Task<GlobalMessage> {
    match input {
        InputEvent::Keyboard(keyboard) => match keyboard {
            keyboard::Event::KeyReleased { .. } => Task::none(),
            _ => Task::none(),
        },
        InputEvent::Mouse(mouse) => match mouse {
            mouse::Event::ButtonReleased(_) => Task::none(),
            _ => Task::none(),
        },
    }
}
