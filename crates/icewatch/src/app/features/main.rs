mod context_menu;
mod dashboard;
mod data;
mod explorer;
mod main_message;
mod toolbar;
mod view;

use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use super::{CONTAINER_PADDING, DEFAULT_THEME};
use crate::{
    app::{
        App,
        message::{InputEvent, Message as GlobalMessage},
        state::FeatureMessage,
    },
    rules::{CriterionKind, Rule},
};

use chrono::{DateTime, Local};
use data::{Criterion, PipelineStage};
use explorer::ExplorerNode;
use iced::{Element, Point, Task, Theme, keyboard, mouse, widget::combo_box, window};
use icewatch_utils::locale::Locale;
use indexmap::IndexMap;
pub(crate) use main_message::{HomeMessage, JournalMessage, RulesMessage, watch_directory_stream};
use view::MainView;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub(crate) is_loading: bool,
    current_view: MainView,
    search_requested: bool,
    search_query: String,
    search_results: IndexMap<PathBuf, ExplorerNode>,
    watch_status_buffer: bool,
    pipeline_queue: VecDeque<PipelineStage>,
    root_registry: Arc<IndexMap<PathBuf, ExplorerNode>>,
    downloaded_count: usize,
    sorted_count: usize,
    indexed_date: DateTime<Local>,
    focused_node: Option<PathBuf>,
    context_menu_visible: bool,
    context_menu_just_opened: bool,
    mouse_position: Point,
    last_mouse_position: Point,
    last_sorted_file: Option<(PathBuf, PathBuf)>,
    focused_rule: Option<usize>,
    sorting_state: combo_box::State<Criterion>,
    active_criterion: Criterion,
    extension_input: Option<String>,
    starts_with_input: Option<String>,
    ends_with_input: Option<String>,
    contains_input: Option<String>,
    destination_input: Option<String>,
    rule_mode: bool,
}

#[derive(Debug, Clone)]
pub struct Context<'a> {
    feature_state: &'a State,
    sorting_rules: &'a Vec<Rule>,
    current_theme: &'a str,
    current_locale: &'a str,
    root_directory: &'a Path,
    themes: &'a HashMap<String, Theme>,
    locales: &'a HashMap<String, Locale>,
    date: &'a DateTime<Local>,
    last_redraw: &'a Duration,
    watch_status: &'a bool,
}

impl<'a> Context<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            sorting_rules: &app.persistent_state.sorting_rules,
            root_directory: app.persistent_state.root_directory.as_path(),
            watch_status: &app.persistent_state.watch_status,
            current_theme: &app.persistent_state.current_theme,
            current_locale: &app.persistent_state.current_locale,
            feature_state: &app.features_state.main,
            date: &app.app_state.date,
            last_redraw: &app.app_state.last_redraw,
            themes: &app.app_state.themes,
            locales: &app.app_state.locales,
        }
    }
}

#[derive(Debug)]
pub struct ContextMut<'a> {
    sorting_rules: &'a mut Vec<Rule>,
    feature_state: &'a mut State,
    root_directory: &'a mut PathBuf,
    watch_status: &'a mut bool,
    overwrite_existing: &'a mut bool,
    sorting_enabled: &'a mut bool,
    purge_empty: &'a mut bool,
}

impl<'a> ContextMut<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self {
            feature_state: &mut app.features_state.main,
            root_directory: &mut app.persistent_state.root_directory,
            watch_status: &mut app.persistent_state.watch_status,
            overwrite_existing: &mut app.persistent_state.overwrite_existing,
            sorting_rules: &mut app.persistent_state.sorting_rules,
            sorting_enabled: &mut app.persistent_state.sorting_enabled,
            purge_empty: &mut app.persistent_state.purge_empty_directories,
        }
    }
}

pub fn init(_ctx: ContextMut<'_>) {}

#[derive(Debug, Clone)]
pub enum Message {
    Home(HomeMessage),
    Rules(RulesMessage),
    Journal(JournalMessage),
    Return,
}

impl From<Message> for GlobalMessage {
    fn from(msg: Message) -> GlobalMessage {
        GlobalMessage::Feature(FeatureMessage::Main(msg))
    }
}

pub(crate) fn update<'a>(msg: Message, ctx: ContextMut<'a>) -> Task<GlobalMessage> {
    match msg {
        Message::Home(msg) => msg.update(ctx),
        Message::Rules(msg) => msg.update(ctx),
        Message::Journal(msg) => msg.update(ctx),
        Message::Return => {
            ctx.feature_state.current_view = MainView::Home;
            Task::none()
        }
    }
}

pub(crate) fn view<'a>(ctx: Context<'a>, _window_id: window::Id) -> Element<'a, GlobalMessage> {
    let current_locale = ctx.current_locale;
    let current_theme = ctx.current_theme;

    let theme = ctx
        .themes
        .get(current_theme)
        .unwrap_or_else(|| ctx.themes.get(DEFAULT_THEME).unwrap_or(&iced::Theme::Dark));

    let locale = ctx.locales.get(current_locale).expect("locale not found");

    ctx.feature_state.current_view.view(ctx.clone(), &locale, &theme)
}

pub(crate) fn input(input: &InputEvent) -> Task<GlobalMessage> {
    match input {
        InputEvent::Keyboard(keyboard::Event::KeyReleased {
            key: keyboard::Key::Named(keyboard::key::Named::F11),
            ..
        }) => window::latest().and_then(|id| {
            window::mode(id).then(move |mode| {
                let next = match mode {
                    window::Mode::Fullscreen => window::Mode::Windowed,
                    _ => window::Mode::Fullscreen,
                };
                window::set_mode(id, next)
            })
        }),

        InputEvent::Mouse(mouse::Event::CursorMoved { position }) => {
            Task::done(HomeMessage::CaptureMousePosition(*position).into())
        }
        InputEvent::Mouse(mouse::Event::ButtonReleased(btn)) => {
            Task::done(HomeMessage::CaptureMouseBtn(*btn).into())
        }

        _ => Task::none(),
    }
}
