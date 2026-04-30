mod context_menu;
mod dashboard;
mod explorer;
mod toolbar;
mod view;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use super::{CONTAINER_PADDING, DEFAULT_THEME};
use crate::{
    app::{
        App,
        message::{AppMessage, InputEvent, Message as GlobalMessage, SystemMessage},
        state::{FeatureMessage, Window},
    },
    rules::{ByExtension, ByName, CriterionKind, Rule},
};

use anyhow::Context as _;
use chrono::{DateTime, Local};
use explorer::ExplorerNode;
use iced::{
    Element, Point, Task, Theme,
    futures::{self, SinkExt},
    keyboard, mouse, stream,
    widget::combo_box,
    window::Id,
};
use icewatch_utils::{command::Command, locale::Locale};
use indexmap::IndexMap;
use notify::{
    EventKind, RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
    recommended_watcher,
};
use smol::stream::StreamExt;
use view::MainView;

#[derive(Debug, Clone, Default)]
pub(crate) enum Criterion {
    ByName,
    #[default]
    ByExtension,
}

impl std::fmt::Display for Criterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Criterion::ByName => write!(f, "By name"),
            Criterion::ByExtension => write!(f, "By extension"),
        }
    }
}

/// Represents the current IO pipeline stage.
#[derive(Debug, Clone, Default)]
pub enum PipelineStage {
    /// Represents the full indexing stage, scanning the entire root directory recursively.
    #[default]
    IndexFull,

    /// Represents a partial indexing stage, scanning only the specified paths.
    IndexPaths(Vec<PathBuf>),

    /// Represents a stage that purges empty directories recursively at the root.
    PurgeEmptyDirs,

    /// Represents a stage that sorts the indexed files according to existing sorting rules and
    /// replaces them in the filesystem to represent the sorted order.
    Sort,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub(crate) is_loading: bool,
    current_view: MainView,
    search_requested: bool,
    search_query: String,
    search_results: IndexMap<PathBuf, ExplorerNode>,
    watch_status_buffer: bool,
    pipeline_queue: VecDeque<PipelineStage>,
    theme_state: combo_box::State<String>,
    locale_state: combo_box::State<String>,
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
    themes: &'a mut HashMap<String, Theme>,
    locales: &'a mut HashMap<String, Locale>,
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
            themes: &mut app.app_state.themes,
            locales: &mut app.app_state.locales,
            root_directory: &mut app.persistent_state.root_directory,
            watch_status: &mut app.persistent_state.watch_status,
            overwrite_existing: &mut app.persistent_state.overwrite_existing,
            sorting_rules: &mut app.persistent_state.sorting_rules,
            sorting_enabled: &mut app.persistent_state.sorting_enabled,
            purge_empty: &mut app.persistent_state.purge_empty_directories,
        }
    }
}

pub fn init(ctx: ContextMut<'_>) {
    let themes = ctx.themes.keys().cloned().collect();
    let locales = ctx.locales.keys().cloned().collect();
    ctx.feature_state.theme_state = combo_box::State::new(themes);
    ctx.feature_state.locale_state = combo_box::State::new(locales);
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Represents a progress update during indexing.
    IndexingProgress(PathBuf, ExplorerNode),

    /// Represents the completion of indexing.
    IndexingComplete {
        indexed_date: DateTime<Local>,
        downloaded_count: usize,
    },

    /// Represents the completion of sorting.
    SortingComplete {
        sorted_count: usize,
        moves: Vec<(PathBuf, PathBuf)>,
    },

    /// Represents the completion of purging.
    PurgeComplete {
        removed: Vec<PathBuf>,
    },

    /// Represents a request to run a partial pipeline. (takes a list of paths to index)
    RunPartialPipeline(Vec<PathBuf>),

    /// Represents a request to run the full pipeline. (indexing all files in the root recursively)
    RunFullPipeline,

    /// Represents a request to advance the pipeline.
    AdvancePipeline,

    /// Represents a request to remove paths from the registry.
    RemovePaths(Vec<PathBuf>),

    /// Represents a request to capture a mouse button press.
    CaptureMouseBtn(mouse::Button),

    /// Represents a request to capture the mouse position.
    CaptureMousePosition(Point),

    /// Represents a request to change the root directory.
    ChangeRoot(Option<PathBuf>),

    /// Represents a request to toggle the context menu.
    ToggleContextMenu(PathBuf),

    /// Represents a request to expand a node.
    ExpandNode(PathBuf),

    /// Represents a request to focus a node.
    FocusNode(PathBuf),

    /// Represents a request to show a node.
    ShowNode,

    /// Represents a request to open a node.
    OpenNode,

    /// Represents an input event from the search bar.
    SearchBarInput(String),

    /// Represents a request to submit the search bar input.
    SearchBarSubmit,

    /// Represents a request to clear the search bar input and abort search.
    SearchClear,

    /// Represents a request to clear the focus.
    ClearFocus,

    /// Represents a request to open the settings.
    OpenSettings,

    /// Represents a request to manage rules.
    OpenRules,

    /// Represents a request to toggle the watch status.
    ToggleWatch,

    OpenJournal,
    ReturnHome,
    FocusRule(Option<usize>),
    RemoveRule(Option<usize>),
    EditRule(Option<usize>),
    SetCriterion(Criterion),
    ApplyRuleEdit(Option<usize>),
    ExtensionInput(String),
    StartsWithInput(String),
    EndsWithInput(String),
    ContainsInput(String),
    DestinationInput(String),
    CancelEdit,
    AddRule,
}

impl From<Message> for GlobalMessage {
    fn from(msg: Message) -> GlobalMessage {
        GlobalMessage::Feature(FeatureMessage::Main(msg))
    }
}

pub(crate) fn update<'a>(msg: Message, ctx: ContextMut<'a>) -> Task<GlobalMessage> {
    match msg {
        // IO Pipeline
        Message::AdvancePipeline => match ctx.feature_state.pipeline_queue.pop_front() {
            Some(PipelineStage::IndexFull) => {
                let root = ctx.root_directory.clone();
                return Task::run(index_directory_stream(root), |msg| msg);
            }
            Some(PipelineStage::IndexPaths(paths)) => {
                let download_count = ctx.feature_state.downloaded_count;
                let root = ctx.root_directory.clone();
                return Task::run(index_paths_stream(root, paths, download_count), |msg| msg);
            }
            Some(PipelineStage::Sort) => {
                let registry = ctx.feature_state.root_registry.clone();
                let root = ctx.root_directory.clone();
                let rules = ctx.sorting_rules.to_vec();
                return Task::perform(
                    sort_directory(root, registry, rules, *ctx.overwrite_existing),
                    |msg| msg,
                );
            }
            Some(PipelineStage::PurgeEmptyDirs) => {
                let registry = ctx.feature_state.root_registry.clone();
                return Task::future(purge_empty_dirs(registry));
            }
            None => {
                ctx.feature_state.is_loading = false;
                *ctx.watch_status = ctx.feature_state.watch_status_buffer;
            }
        },
        Message::RunPartialPipeline(paths) => {
            ctx.feature_state.watch_status_buffer = ctx.watch_status.clone();
            *ctx.watch_status = false;

            ctx.feature_state.is_loading = true;

            let mut queue = VecDeque::new();
            queue.push_back(PipelineStage::IndexPaths(paths));
            if *ctx.sorting_enabled {
                queue.push_back(PipelineStage::Sort);
            }
            if *ctx.purge_empty {
                queue.push_back(PipelineStage::PurgeEmptyDirs);
            }

            ctx.feature_state.pipeline_queue = queue;
            return Task::done(Message::AdvancePipeline.into());
        }
        Message::RunFullPipeline => {
            ctx.feature_state.watch_status_buffer = ctx.watch_status.clone();
            *ctx.watch_status = false;

            ctx.feature_state.is_loading = true;
            Arc::make_mut(&mut ctx.feature_state.root_registry).clear();

            ctx.feature_state.downloaded_count = 0;

            let mut queue = VecDeque::new();
            queue.push_back(PipelineStage::IndexFull);

            if *ctx.sorting_enabled {
                queue.push_back(PipelineStage::Sort);
            }

            if *ctx.purge_empty {
                queue.push_back(PipelineStage::PurgeEmptyDirs);
            }

            ctx.feature_state.pipeline_queue = queue;
            return Task::done(Message::AdvancePipeline.into());
        }
        Message::IndexingProgress(path, node) => {
            Arc::make_mut(&mut ctx.feature_state.root_registry).insert(path, node);
        }
        Message::IndexingComplete { indexed_date, downloaded_count } => {
            ctx.feature_state.indexed_date = indexed_date;
            ctx.feature_state.downloaded_count = downloaded_count;
            return Task::done(Message::AdvancePipeline.into());
        }
        Message::SortingComplete { sorted_count, moves } => {
            apply_moves(Arc::make_mut(&mut ctx.feature_state.root_registry), &moves);
            ctx.feature_state.sorted_count += sorted_count;

            if !moves.is_empty()
                && let Some((old_path, new_path)) = moves.last()
            {
                let old_path = old_path.strip_prefix(&ctx.root_directory).unwrap_or(&old_path);
                let new_path = new_path.strip_prefix(&ctx.root_directory).unwrap_or(&new_path);
                ctx.feature_state.last_sorted_file =
                    Some((old_path.to_path_buf(), new_path.to_path_buf()));
            }

            return Task::done(Message::AdvancePipeline.into());
        }
        Message::PurgeComplete { removed } => {
            let registry_mut = Arc::make_mut(&mut ctx.feature_state.root_registry);
            for path in &removed {
                registry_mut.shift_remove(path);

                if let Some(parent) = path.parent() {
                    if let Some(parent_node) = registry_mut.get_mut(parent) {
                        parent_node.children.retain(|c| c != path);
                    }
                }
            }
            return Task::done(Message::AdvancePipeline.into());
        }
        Message::RemovePaths(paths) => {
            let registry_mut = Arc::make_mut(&mut ctx.feature_state.root_registry);
            for path in &paths {
                registry_mut.shift_remove(path);
                if let Some(parent) = path.parent() {
                    if let Some(parent_node) = registry_mut.get_mut(parent) {
                        parent_node.children.retain(|c| c != path);
                    }
                }
            }
        }

        // Controls
        Message::ChangeRoot(new_root) => {
            if let Some(dir) = new_root {
                *ctx.root_directory = dir;
            }
            return Task::done(Message::RunFullPipeline.into());
        }
        Message::ToggleWatch => {
            *ctx.watch_status = !*ctx.watch_status;
        }

        // UI Functionality
        Message::SearchClear => {
            ctx.feature_state.search_query = String::new();
            ctx.feature_state.search_requested = false;
            ctx.feature_state.search_results.clear();
        }
        Message::SearchBarInput(i) => {
            ctx.feature_state.search_query = i;
        }
        Message::SearchBarSubmit => {
            let term = ctx.feature_state.search_query.to_ascii_lowercase();
            ctx.feature_state.search_requested = true;

            let mut results: IndexMap<PathBuf, ExplorerNode> = IndexMap::new();

            for (path, node) in ctx.feature_state.root_registry.iter() {
                let matches = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map_or(false, |name| name.to_ascii_lowercase().contains(&term));

                if !matches {
                    continue;
                }

                // Insert the matched node itself
                results.entry(path.clone()).or_insert_with(|| node.clone());

                // Ensure its parent exists in results with this path in its children
                if let Some(parent_path) = path.parent()
                    && let Some(parent_node) = ctx.feature_state.root_registry.get(parent_path)
                {
                    let parent = results.entry(parent_path.to_path_buf()).or_insert_with(|| {
                        ExplorerNode { children: vec![], expanded: true, ..parent_node.clone() }
                    });
                    if !parent.children.contains(path) {
                        parent.children.push(path.clone());
                    }
                }
            }

            ctx.feature_state.search_results = results;
        }
        Message::OpenRules => {
            ctx.feature_state.current_view = MainView::Rules;
        }
        Message::OpenSettings => {
            return Task::done(GlobalMessage::App(AppMessage::View(Window::Settings)));
        }
        Message::OpenJournal => {
            ctx.feature_state.current_view = MainView::Journal;
        }
        Message::OpenNode => {
            let cmd = cfg!(target_os = "windows").then(|| "explorer").unwrap_or_else(|| "open");
            if let Some(node) = ctx.feature_state.focused_node.clone() {
                return Task::done(GlobalMessage::System(SystemMessage::Execute(
                    Command::new(cmd).arg(node.to_string_lossy().into_owned()),
                )));
            }
        }
        Message::ShowNode => {
            if let Some(node) = ctx.feature_state.focused_node.clone() {
                if cfg!(target_os = "windows") {
                    let cmd = Command::new("explorer")
                        .arg("/select,")
                        .arg(node.to_string_lossy().into_owned());
                    return Task::done(GlobalMessage::System(SystemMessage::Execute(cmd)));
                } else {
                    let cmd = Command::new("xdg-open").arg(
                        node.parent().expect("path has no parent").to_string_lossy().into_owned(),
                    );
                    return Task::done(GlobalMessage::System(SystemMessage::Execute(cmd)));
                }
            }
        }
        Message::ExpandNode(key) => {
            if let Some(node) = Arc::make_mut(&mut ctx.feature_state.root_registry).get_mut(&key) {
                node.expanded = !node.expanded;
            }
        }
        Message::FocusNode(path) => {
            ctx.feature_state.focused_node = Some(path);
        }
        Message::ClearFocus => {
            if !ctx.feature_state.context_menu_visible {
                ctx.feature_state.focused_node = None;
            }
        }
        Message::ToggleContextMenu(node_path) => {
            let old_state = ctx.feature_state.context_menu_visible;

            if !old_state {
                ctx.feature_state.context_menu_just_opened = true;
            }

            ctx.feature_state.context_menu_visible = !old_state;
            ctx.feature_state.last_mouse_position = ctx.feature_state.mouse_position;
            ctx.feature_state.focused_node = Some(node_path);
        }

        // Input Handling
        Message::CaptureMouseBtn(btn) => match btn {
            _ => {
                let ctx_menu_just_opened = ctx.feature_state.context_menu_just_opened;
                if ctx_menu_just_opened {
                    ctx.feature_state.context_menu_just_opened = false;
                } else {
                    ctx.feature_state.context_menu_visible = false;
                }
            }
        },
        Message::CaptureMousePosition(pos) => {
            ctx.feature_state.mouse_position = pos;
        }

        Message::FocusRule(idx) => {
            ctx.feature_state.focused_rule = idx;
        }
        Message::RemoveRule(Some(idx)) => {
            ctx.sorting_rules.remove(idx);
        }
        Message::SetCriterion(c) => {
            ctx.feature_state.active_criterion = c;
        }
        Message::CancelEdit => {
            ctx.feature_state.rule_mode = false;
        }
        Message::EditRule(Some(idx)) => {
            if let Some(rule) = ctx.sorting_rules.get(idx) {
                match &rule.criterion {
                    CriterionKind::ByExtension(crit) => {
                        ctx.feature_state.extension_input = Some(crit.extensions.join(", "));
                        ctx.feature_state.active_criterion = Criterion::ByExtension;
                    }
                    CriterionKind::ByName(crit) => {
                        ctx.feature_state.starts_with_input = crit.starts_with.clone();
                        ctx.feature_state.ends_with_input = crit.ends_with.clone();
                        ctx.feature_state.contains_input = crit.contains.clone();
                        ctx.feature_state.active_criterion = Criterion::ByName;
                    }
                }
                ctx.feature_state.destination_input =
                    Some(rule.destination.to_string_lossy().into_owned());
                ctx.feature_state.rule_mode = true;
            }
        }
        Message::DestinationInput(dest) => ctx.feature_state.destination_input = Some(dest),
        Message::ExtensionInput(ext) => ctx.feature_state.extension_input = Some(ext),
        Message::StartsWithInput(s) => ctx.feature_state.starts_with_input = Some(s),
        Message::EndsWithInput(s) => ctx.feature_state.ends_with_input = Some(s),
        Message::ContainsInput(s) => ctx.feature_state.contains_input = Some(s),
        Message::ApplyRuleEdit(idx) => {
            let fs = ctx.feature_state;
            let rule = match &fs.active_criterion {
                Criterion::ByExtension => Rule::new(
                    ByExtension::new(fs.extension_input.clone().unwrap_or_default()),
                    &fs.destination_input.clone().unwrap_or_default(),
                ),
                Criterion::ByName => Rule::new(
                    ByName {
                        starts_with: fs.starts_with_input.clone(),
                        ends_with: fs.ends_with_input.clone(),
                        contains: fs.contains_input.clone(),
                    },
                    &fs.destination_input.clone().unwrap_or_default(),
                ),
            }
            .context("failed to create rule")
            .unwrap();
            match idx.and_then(|i| ctx.sorting_rules.get_mut(i)) {
                Some(existing) => *existing = rule,
                None => ctx.sorting_rules.push(rule),
            }
            fs.rule_mode = false;
        }
        Message::ReturnHome => {
            ctx.feature_state.current_view = MainView::Home;
        }
        Message::AddRule => {
            ctx.feature_state.extension_input = None;
            ctx.feature_state.starts_with_input = None;
            ctx.feature_state.ends_with_input = None;
            ctx.feature_state.contains_input = None;
            ctx.feature_state.destination_input = None;
            ctx.feature_state.focused_rule = None;
            ctx.feature_state.rule_mode = true;
        }
        // exhaustive: RemoveRule(None), EditRule(None) are no-ops
        _ => {}
    }
    Task::none()
}

pub(crate) fn view<'a>(ctx: Context<'a>, _window_id: Id) -> Element<'a, GlobalMessage> {
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
        InputEvent::Keyboard(keyboard) => match keyboard {
            keyboard::Event::KeyReleased { key, .. } => match key {
                keyboard::Key::Named(keyboard::key::Named::F11) => {
                    iced::window::latest().and_then(|id| {
                        iced::window::mode(id).then(move |mode| {
                            if let iced::window::Mode::Fullscreen = mode {
                                iced::window::set_mode(id, iced::window::Mode::Windowed)
                            } else {
                                iced::window::set_mode(id, iced::window::Mode::Fullscreen)
                            }
                        })
                    })
                }
                _ => Task::none(),
            },
            _ => Task::none(),
        },
        InputEvent::Mouse(mouse) => match mouse {
            mouse::Event::CursorMoved { position } => {
                Task::done(Message::CaptureMousePosition(position.clone()).into())
            }
            mouse::Event::ButtonReleased(btn) => {
                Task::done(Message::CaptureMouseBtn(btn.clone()).into())
            }
            _ => Task::none(),
        },
    }
}

fn index_paths_stream(
    root: PathBuf,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
    downloaded_count: usize,
) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, move |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let indexed_date = Local::now();
        let paths = paths.into_iter().map(|p| p.as_ref().to_path_buf()).collect::<Vec<PathBuf>>();
        let mut stack = VecDeque::from(paths);
        let mut downloaded_count = downloaded_count;

        while let Some(path) = stack.pop_front() {
            let metadata = smol::fs::metadata(&path).await;

            let mut is_directory = false;
            let mut not_available = true;
            let mut created_at = Local::now();
            let mut size_bytes = 0u64;

            if let Ok(metadata) = metadata
                && let Ok(created) = metadata.created()
            {
                is_directory = metadata.is_dir();
                not_available = false;
                size_bytes = metadata.len();
                created_at = DateTime::<Local>::from(created);
                if created_at.date_naive() == indexed_date.date_naive() {
                    downloaded_count += 1;
                }
            }

            let mut children = vec![];
            if is_directory {
                if let Ok(mut entries) = smol::fs::read_dir(&path).await {
                    while let Some(Ok(entry)) = entries.next().await {
                        children.push(entry.path());
                    }
                    stack.extend(children.iter().cloned());
                }
            }

            if path != root {
                let node = ExplorerNode {
                    size_bytes,
                    unidentified: not_available,
                    created: created_at,
                    path: path.clone(),
                    is_dir: is_directory,
                    expanded: false,
                    children,
                };
                let _ = tx.send(Message::IndexingProgress(path, node).into()).await;
            }
        }

        let _ = tx.send(Message::IndexingComplete { indexed_date, downloaded_count }.into()).await;
    })
}

fn index_directory_stream(root: PathBuf) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let indexed_date = Local::now();
        let mut stack = VecDeque::from([root.clone()]);
        let mut downloaded_count = 0usize;

        while let Some(path) = stack.pop_front() {
            let metadata = smol::fs::metadata(&path).await;

            let mut is_directory = false;
            let mut not_available = true;
            let mut created_at = Local::now();
            let mut size_bytes = 0u64;

            if let Ok(metadata) = metadata
                && let Ok(created) = metadata.created()
            {
                is_directory = metadata.is_dir();
                not_available = false;
                size_bytes = metadata.len();
                created_at = DateTime::<Local>::from(created);
                if created_at.date_naive() == indexed_date.date_naive() {
                    downloaded_count += 1;
                }
            }

            let mut children = vec![];
            if is_directory {
                if let Ok(mut entries) = smol::fs::read_dir(&path).await {
                    while let Some(Ok(entry)) = entries.next().await {
                        children.push(entry.path());
                    }
                    stack.extend(children.iter().cloned());
                }
            }

            if path != root {
                let node = ExplorerNode {
                    size_bytes,
                    unidentified: not_available,
                    created: created_at,
                    path: path.clone(),
                    is_dir: is_directory,
                    expanded: false,
                    children,
                };
                let _ = tx.send(Message::IndexingProgress(path, node).into()).await;
            }
        }

        let _ = tx.send(Message::IndexingComplete { indexed_date, downloaded_count }.into()).await;
    })
}

fn apply_moves(registry: &mut IndexMap<PathBuf, ExplorerNode>, moves: &[(PathBuf, PathBuf)]) {
    for (old_path, new_path) in moves {
        // Update the node itself
        if let Some(mut node) = registry.shift_remove(old_path) {
            node.path = new_path.clone();
            registry.insert(new_path.clone(), node);
        }

        // Remove from old parent's children list
        if let Some(old_parent) = old_path.parent() {
            if let Some(parent_node) = registry.get_mut(old_parent) {
                parent_node.children.retain(|c| c != old_path);
            }
        }

        // Add to new parent's children list, inserting the parent node if missing
        if let Some(new_parent) = new_path.parent() {
            if !registry.contains_key(new_parent) {
                registry.insert(
                    new_parent.to_path_buf(),
                    ExplorerNode {
                        path: new_parent.to_path_buf(),
                        is_dir: true,
                        expanded: false,
                        unidentified: false,
                        children: vec![new_path.clone()],
                        created: Local::now(),
                        size_bytes: 0,
                    },
                );
            } else if let Some(parent_node) = registry.get_mut(new_parent) {
                if !parent_node.children.contains(new_path) {
                    parent_node.children.push(new_path.clone());
                }
            }
        }
    }
}

pub(crate) fn watch_directory_stream(root: PathBuf) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let (notify_tx, notify_rx) = smol::channel::unbounded();

        // Watcher runs in a separate thread, once the thread receives an event,
        // it blocks until the event is sent to the notify channel.
        let mut watcher = recommended_watcher(move |event| {
            let _ = smol::block_on(notify_tx.send(event));
        })
        .expect("failed to create watcher");

        watcher.watch(&root, RecursiveMode::Recursive).expect("failed to watch directory");

        // Main thread receives events from the notify channel and sends them to the stream.
        // If the channel is empty, the async executor will suspend the task until an event arrives.
        // If it receives an event, it sends it to the stream.
        while let Ok(Ok(event)) = notify_rx.recv().await {
            let msg = match event.kind {
                EventKind::Create(_) => Some(Message::RunPartialPipeline(event.paths)),
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    Some(Message::RemovePaths(event.paths))
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    Some(Message::RunPartialPipeline(event.paths))
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                    // Can't tell direction — check if path exists to decide
                    let path = &event.paths[0];
                    if path.exists() {
                        Some(Message::RunPartialPipeline(event.paths))
                    } else {
                        Some(Message::RemovePaths(event.paths))
                    }
                }
                EventKind::Remove(_) => Some(Message::RemovePaths(event.paths)),
                _ => None,
            };

            if let Some(msg) = msg {
                let _ = tx.send(msg.into()).await;
            }
        }
    })
}

async fn sort_directory(
    root: PathBuf,
    registry: Arc<IndexMap<PathBuf, ExplorerNode>>,
    rules: Vec<Rule>,
    overwrite: bool,
) -> GlobalMessage {
    let mut sorted = IndexMap::with_capacity(registry.len());
    let mut new_paths = HashSet::with_capacity(registry.len());

    for (_, node) in registry.iter() {
        for rule in &rules {
            if rule.applies_to(&node.path) && !node.unidentified && !sorted.contains_key(&node.path)
            {
                let Some(name) = node.path.file_name().and_then(|n| n.to_str()) else {
                    tracing::error!("could not get file name for {:?}", node.path);
                    continue;
                };

                let dest_parent = root.join(&rule.destination);
                let dest_path = dest_parent.join(name);
                if dest_path == node.path {
                    continue;
                }

                if !overwrite && new_paths.contains(&dest_path) {
                    tracing::warn!("skipping duplicate destination: {:?}", dest_path);
                    continue;
                }

                let Ok(_) = smol::fs::create_dir_all(&dest_parent).await else {
                    tracing::error!("failed to create destination directory for {:?}", dest_path);
                    continue;
                };
                let Ok(_) = smol::fs::rename(&node.path, &dest_path).await else {
                    tracing::error!("failed to rename file {:?} to {:?}", node.path, dest_path);
                    continue;
                };

                sorted.insert(node.path.clone(), dest_path.clone());
                new_paths.insert(dest_path);
                break;
            }
        }
    }
    Message::SortingComplete { sorted_count: sorted.len(), moves: sorted.into_iter().collect() }
        .into()
}

async fn purge_empty_dirs(registry: Arc<IndexMap<PathBuf, ExplorerNode>>) -> GlobalMessage {
    let mut dirs: Vec<&PathBuf> =
        registry.iter().filter(|(_, node)| node.is_dir).map(|(path, _)| path).collect();

    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    let mut removed: HashSet<&PathBuf> = HashSet::new();

    for path in dirs {
        let Some(node) = registry.get(path) else { continue };
        let all_children_removed = node.children.iter().all(|child| removed.contains(child));

        if node.children.is_empty() || all_children_removed {
            let _ = smol::fs::remove_dir(path).await;
            removed.insert(path);
        }
    }

    Message::PurgeComplete { removed: removed.into_iter().cloned().collect() }.into()
}
