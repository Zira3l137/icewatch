use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Local;
use iced::Point;
use iced::Task;
use iced::futures;
use iced::futures::SinkExt;
use iced::mouse;
use iced::stream;
use icewatch_utils::command::Command;
use indexmap::IndexMap;
use notify::EventKind;
use notify::RecursiveMode;
use notify::Watcher;
use notify::event::CreateKind;
use notify::event::ModifyKind;
use notify::event::RenameMode;
use notify::recommended_watcher;
use smol::stream::StreamExt;

use crate::app::Window;
use crate::app::features::main::ContextMut;
use crate::app::features::main::Message;
use crate::app::features::main::data::DOWNLOAD_TEMP_EXTENSIONS;
use crate::app::features::main::data::PipelineStage;
use crate::app::features::main::data::WatcherEvent;
use crate::app::features::main::elements::explorer::ExplorerNode;
use crate::app::features::main::view::MainView;
use crate::app::message::AppMessage;
use crate::app::message::Message as GlobalMessage;
use crate::app::message::SystemMessage;
use crate::journal::Action;
use crate::journal::ActionType;
use crate::rules::Rule;

/// Represents a message from the home view.
#[derive(Debug, Clone)]
pub(crate) enum HomeMessage {
    /// Represents a progress update during indexing.
    IndexingProgress { path: PathBuf, node: ExplorerNode, is_downloaded: bool },

    /// Represents the completion of indexing.
    IndexingComplete { indexed_date: DateTime<Local>, downloaded_count: usize },

    /// Represents the progress of sorting. Contains a pair of path buffers representing the old and new paths of a file being moved.
    SortingProgress((PathBuf, PathBuf)),

    /// Represents the completion of sorting.
    SortingComplete,

    /// Represents a file system event caught by the watcher.
    WatcherEvent(WatcherEvent),

    /// Represents the progress of purging.
    PurgeProgress(PathBuf),

    /// Represents the completion of purging.
    PurgeComplete,

    /// Represents a request to run a partial pipeline. (takes a list of paths to index)
    RunPartialPipeline(Vec<(PathBuf, bool)>),

    /// Represents a request to run the full pipeline. (indexing all files in the root recursively)
    RunFullPipeline,

    /// Represents a request to advance the pipeline.
    AdvancePipeline,

    /// Represents a request to remove paths from the registry.
    RemovePaths { paths: Vec<PathBuf>, action_type: ActionType },

    /// Represents a request to capture a mouse button press.
    #[expect(dead_code)]
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

    /// Represents a request to clear the search bar input and abort search.
    SearchClear,

    /// Represents a request to clear the focus.
    ClearFocus,

    /// Represents a request to switch to the settings view.
    OpenSettings,

    /// Represents a request to switch to the rules view.
    OpenRules,

    /// Represents a request to switch to the journal view.
    OpenJournal,

    /// Represents a request to toggle the watch status.
    ToggleWatch,
}

impl From<HomeMessage> for GlobalMessage {
    fn from(msg: HomeMessage) -> Self {
        Message::Home(msg).into()
    }
}

impl HomeMessage {
    pub(crate) fn update<'a>(self, ctx: ContextMut<'a>) -> Task<GlobalMessage> {
        match self {
            // IO Pipeline
            HomeMessage::AdvancePipeline => match ctx.feature_state.pipeline_queue.pop_front() {
                Some(PipelineStage::IndexFull) => {
                    let root = ctx.root_directory.clone();
                    return Task::run(index_directory_stream(root), |msg| msg);
                }
                Some(PipelineStage::IndexPaths(paths)) => {
                    let download_count = ctx.feature_state.downloaded_count;
                    let root = ctx.root_directory.clone();
                    return Task::stream(index_paths_stream(root, paths, download_count));
                }
                Some(PipelineStage::Sort) => {
                    let registry = ctx.feature_state.root_registry.clone();
                    let root = ctx.root_directory.clone();
                    let rules = ctx.sorting_rules.to_vec();
                    let overwrite = *ctx.overwrite_existing;
                    return Task::stream(sort_directory(root, registry, rules, overwrite));
                }
                Some(PipelineStage::PurgeEmptyDirs) => {
                    let registry = ctx.feature_state.root_registry.clone();
                    return Task::stream(purge_empty_dirs(registry));
                }
                None => {
                    ctx.feature_state.is_loading = false;
                    *ctx.watch_status = ctx.feature_state.watch_status_buffer;
                }
            },
            HomeMessage::RunPartialPipeline(paths) => {
                ctx.feature_state.watch_status_buffer = *ctx.watch_status;
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
                return Task::done(HomeMessage::AdvancePipeline.into());
            }
            HomeMessage::RunFullPipeline => {
                ctx.feature_state.watch_status_buffer = *ctx.watch_status;
                *ctx.watch_status = false;
                ctx.feature_state.is_loading = true;
                ctx.feature_state.downloaded_count = 0;

                Arc::make_mut(&mut ctx.feature_state.root_registry).clear();

                let mut queue = VecDeque::new();
                queue.push_back(PipelineStage::IndexFull);

                if *ctx.sorting_enabled {
                    queue.push_back(PipelineStage::Sort);
                }

                if *ctx.purge_empty {
                    queue.push_back(PipelineStage::PurgeEmptyDirs);
                }

                ctx.feature_state.pipeline_queue = queue;
                return Task::done(HomeMessage::AdvancePipeline.into());
            }
            HomeMessage::IndexingProgress { path, node, is_downloaded } => {
                Arc::make_mut(&mut ctx.feature_state.root_registry).insert(path, node);
                if is_downloaded {
                    ctx.feature_state.downloaded_count += 1;
                }
            }
            HomeMessage::IndexingComplete { indexed_date, downloaded_count } => {
                ctx.feature_state.indexed_date = indexed_date;
                ctx.feature_state.downloaded_count = downloaded_count;
                return Task::done(HomeMessage::AdvancePipeline.into());
            }
            HomeMessage::SortingProgress(move_pair) => {
                let (source, destination) = move_pair.clone();

                apply_moves(
                    Arc::make_mut(&mut ctx.feature_state.root_registry),
                    std::slice::from_ref(&move_pair),
                );
                ctx.feature_state.moved.push(move_pair);
                ctx.journal.log(Action::Moved { source, destination }, ActionType::Automatic);
            }
            HomeMessage::SortingComplete => {
                let moves = &ctx.feature_state.moved;
                if !moves.is_empty()
                    && let Some((old_path, new_path)) = moves.last()
                {
                    let old_path = old_path.strip_prefix(&ctx.root_directory).unwrap_or(old_path);
                    let new_path = new_path.strip_prefix(ctx.root_directory).unwrap_or(new_path);
                    ctx.feature_state.last_sorted_file =
                        Some((old_path.to_path_buf(), new_path.to_path_buf()));
                }

                return Task::done(HomeMessage::AdvancePipeline.into());
            }
            HomeMessage::PurgeProgress(removed) => {
                let registry_mut = Arc::make_mut(&mut ctx.feature_state.root_registry);
                registry_mut.shift_remove(&removed);
                if let Some(parent) = removed.parent()
                    && let Some(parent_node) = registry_mut.get_mut(parent)
                {
                    parent_node.children.retain(|c| c != &removed);
                }
                ctx.journal.log(Action::Removed(removed), ActionType::Automatic);
            }
            HomeMessage::PurgeComplete => {
                return Task::done(HomeMessage::AdvancePipeline.into());
            }
            HomeMessage::RemovePaths { paths, action_type } => {
                let registry_mut = Arc::make_mut(&mut ctx.feature_state.root_registry);
                for path in &paths {
                    ctx.journal.log(Action::Removed(path.clone()), action_type.clone());
                    registry_mut.shift_remove(path);
                    if let Some(parent) = path.parent()
                        && let Some(parent_node) = registry_mut.get_mut(parent)
                    {
                        parent_node.children.retain(|c| c != path);
                    }
                }
            }
            HomeMessage::WatcherEvent(event) => match event {
                WatcherEvent::Removed(paths) => {
                    return Task::done(
                        HomeMessage::RemovePaths { paths, action_type: ActionType::Manual }.into(),
                    );
                }
                WatcherEvent::Created(paths) => {
                    paths.iter().filter(|(_, is_dwnld)| *is_dwnld).for_each(|(p, _)| {
                        ctx.journal.log(Action::Downloaded(p.clone()), ActionType::Manual);
                    });
                    return Task::done(HomeMessage::RunPartialPipeline(paths).into());
                }
                WatcherEvent::Renamed(paths) => {
                    apply_moves(Arc::make_mut(&mut ctx.feature_state.root_registry), &paths);
                    paths.iter().for_each(|(from, to)| {
                        ctx.journal.log(
                            Action::Renamed { source: from.clone(), destination: to.clone() },
                            ActionType::Manual,
                        );
                    });
                }
            },

            // Controls
            HomeMessage::ChangeRoot(new_root) => {
                if let Some(dir) = new_root {
                    *ctx.root_directory = dir;
                }
                return Task::done(HomeMessage::RunFullPipeline.into());
            }
            HomeMessage::ToggleWatch => {
                *ctx.watch_status = !*ctx.watch_status;
            }

            // UI Functionality
            HomeMessage::SearchClear => {
                ctx.feature_state.search_query = String::new();
                ctx.feature_state.search_requested = false;
                ctx.feature_state.search_results.clear();
            }
            HomeMessage::SearchBarInput(term) => {
                ctx.feature_state.search_query = term.clone();
                if !term.trim().is_empty() {
                    ctx.feature_state.search_requested = true;
                    let mut results: IndexMap<PathBuf, ExplorerNode> = IndexMap::new();

                    for (path, node) in ctx.feature_state.root_registry.iter() {
                        let matches = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .is_some_and(|name| name.to_ascii_lowercase().contains(&term));

                        if !matches {
                            continue;
                        }

                        // Insert the matched node itself
                        results.entry(path.clone()).or_insert_with(|| node.clone());

                        // Ensure its parent exists in results with this path in its children
                        if let Some(parent_path) = path.parent()
                            && let Some(parent_node) =
                                ctx.feature_state.root_registry.get(parent_path)
                        {
                            let parent =
                                results.entry(parent_path.to_path_buf()).or_insert_with(|| {
                                    ExplorerNode {
                                        children: vec![],
                                        expanded: true,
                                        ..parent_node.clone()
                                    }
                                });
                            if !parent.children.contains(path) {
                                parent.children.push(path.clone());
                            }
                        }
                    }

                    ctx.feature_state.search_results = results;
                } else {
                    ctx.feature_state.search_requested = false;
                    ctx.feature_state.search_results.clear();
                }
            }
            HomeMessage::OpenRules => {
                ctx.feature_state.current_view = MainView::Rules;
            }
            HomeMessage::OpenSettings => {
                return Task::done(GlobalMessage::App(AppMessage::View(Window::Settings)));
            }
            HomeMessage::OpenJournal => {
                ctx.feature_state.current_view = MainView::Journal;
            }
            HomeMessage::OpenNode => {
                let cmd = cfg!(target_os = "windows").then(|| "explorer").unwrap_or("open");
                if let Some(node) = ctx.feature_state.focused_node.clone() {
                    return Task::done(GlobalMessage::System(SystemMessage::Execute(
                        Command::new(cmd).arg(node.to_string_lossy().into_owned()),
                    )));
                }
            }
            HomeMessage::ShowNode => {
                if let Some(node) = ctx.feature_state.focused_node.clone() {
                    if cfg!(target_os = "windows") {
                        let cmd = Command::new("explorer")
                            .arg("/select,")
                            .arg(node.to_string_lossy().into_owned());
                        return Task::done(GlobalMessage::System(SystemMessage::Execute(cmd)));
                    } else {
                        let cmd = Command::new("xdg-open").arg(
                            node.parent()
                                .expect("path has no parent")
                                .to_string_lossy()
                                .into_owned(),
                        );
                        return Task::done(GlobalMessage::System(SystemMessage::Execute(cmd)));
                    }
                }
            }
            HomeMessage::ExpandNode(key) => {
                if let Some(node) =
                    Arc::make_mut(&mut ctx.feature_state.root_registry).get_mut(&key)
                {
                    node.expanded = !node.expanded;
                }
            }
            HomeMessage::FocusNode(path) => {
                ctx.feature_state.focused_node = Some(path);
            }
            HomeMessage::ClearFocus => {
                if !ctx.feature_state.context_menu_visible {
                    ctx.feature_state.focused_node = None;
                }
            }
            HomeMessage::ToggleContextMenu(node_path) => {
                let old_state = ctx.feature_state.context_menu_visible;

                if !old_state {
                    ctx.feature_state.context_menu_just_opened = true;
                }

                ctx.feature_state.context_menu_visible = !old_state;
                ctx.feature_state.last_mouse_position = ctx.feature_state.mouse_position;
                ctx.feature_state.focused_node = Some(node_path);
            }

            // Input Handling
            HomeMessage::CaptureMouseBtn(_) => {
                let ctx_menu_just_opened = ctx.feature_state.context_menu_just_opened;
                if ctx_menu_just_opened {
                    ctx.feature_state.context_menu_just_opened = false;
                } else {
                    ctx.feature_state.context_menu_visible = false;
                }
            }
            HomeMessage::CaptureMousePosition(pos) => {
                ctx.feature_state.mouse_position = pos;
            }
        }
        Task::none()
    }
}

fn index_paths_stream(
    root: PathBuf,
    paths: impl IntoIterator<Item = (PathBuf, bool)>,
    downloaded_count: usize,
) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, move |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let indexed_date = Local::now();
        let paths = paths.into_iter().collect::<Vec<(PathBuf, bool)>>();
        let mut stack = VecDeque::from(paths);
        let mut downloaded_count = downloaded_count;

        while let Some((path, is_downloaded)) = stack.pop_front() {
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
                if is_downloaded {
                    downloaded_count += 1;
                }
            }

            let mut children = vec![];
            if is_directory && let Ok(mut entries) = smol::fs::read_dir(&path).await {
                while let Some(Ok(entry)) = entries.next().await {
                    children.push(entry.path());
                }
                stack.extend(children.iter().cloned().map(|p| (p, is_downloaded)));
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
                let _ = tx
                    .send(HomeMessage::IndexingProgress { path, node, is_downloaded }.into())
                    .await;
            }
        }

        let _ =
            tx.send(HomeMessage::IndexingComplete { indexed_date, downloaded_count }.into()).await;
    })
}

fn index_directory_stream(root: PathBuf) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let indexed_date = Local::now();
        let mut stack = VecDeque::from([root.clone()]);

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
            }

            let mut children = vec![];
            if is_directory && let Ok(mut entries) = smol::fs::read_dir(&path).await {
                while let Some(Ok(entry)) = entries.next().await {
                    children.push(entry.path());
                }
                stack.extend(children.iter().cloned());
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
                let _ = tx
                    .send(HomeMessage::IndexingProgress { path, node, is_downloaded: false }.into())
                    .await;
            }
        }

        let _ = tx
            .send(HomeMessage::IndexingComplete { indexed_date, downloaded_count: 0 }.into())
            .await;
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
        if let Some(old_parent) = old_path.parent()
            && let Some(parent_node) = registry.get_mut(old_parent)
        {
            parent_node.children.retain(|c| c != old_path);
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
            } else if let Some(parent_node) = registry.get_mut(new_parent)
                && !parent_node.children.contains(new_path)
            {
                parent_node.children.push(new_path.clone());
            }
        }
    }
}

pub(crate) fn watch_directory_stream(root: PathBuf) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let (notify_tx, notify_rx) = smol::channel::unbounded();

        let mut watcher = recommended_watcher(move |event| {
            let _ = smol::block_on(notify_tx.send(event));
        })
        .expect("failed to create watcher");

        watcher.watch(&root, RecursiveMode::Recursive).expect("failed to watch directory");

        // Temp paths currently being written to by a browser.
        // Removed on RenameFrom (download finished) or Remove (cancelled).
        let mut pending_downloads: HashSet<PathBuf> = HashSet::new();

        // Graduated temp paths awaiting their final RenameTo.
        // FIFO: the OS always delivers From -> To pairs in causal order,
        // so positional matching should be correct.
        let mut completed_downloads: VecDeque<PathBuf> = VecDeque::new();

        // Zero-byte placeholder paths (Chrome-style pre-allocation).
        // Silently dropped on Remove.
        let mut reserved_names: HashSet<PathBuf> = HashSet::new();

        // RenameFrom paths awaiting a matching RenameTo.
        // Flushed as Removed at the top of the next event if still unmatched
        // (meaning the file was moved out of the watched directory).
        let mut pending_rename_from: Vec<PathBuf> = Vec::new();

        while let Ok(Ok(event)) = notify_rx.recv().await {
            // Flush unmatched RenameFroms from the *previous* event.
            // By the time any new event arrives, a From with no To is
            // definitively a move-out, not a split From/To pair.
            if !pending_rename_from.is_empty() {
                let moved_out = std::mem::take(&mut pending_rename_from);
                let _ = tx
                    .send(HomeMessage::WatcherEvent(WatcherEvent::Removed(moved_out)).into())
                    .await;
            }

            match event.kind {
                // ── Create ───────────────────────────────────────────────────
                EventKind::Create(CreateKind::Any) => {
                    let mut created = Vec::new();

                    for path in event.paths {
                        if is_download_temp(&path) {
                            pending_downloads.insert(path);
                            continue;
                        }

                        let is_empty =
                            std::fs::metadata(&path).map(|m| m.len() == 0).unwrap_or(true);

                        if is_empty {
                            // Chrome-style zero-byte placeholder; track and skip.
                            reserved_names.insert(path);
                            continue;
                        }

                        created.push((path, false));
                    }

                    if !created.is_empty() {
                        let _ = tx
                            .send(HomeMessage::WatcherEvent(WatcherEvent::Created(created)).into())
                            .await;
                    }
                }

                // ── Rename/From ──────────────────────────────────────────────
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    for path in event.paths {
                        if pending_downloads.remove(&path) {
                            // Download temp graduated; await the matching To.
                            completed_downloads.push_back(path);
                        } else {
                            // Real file; await the matching To.
                            pending_rename_from.push(path);
                        }
                    }
                }

                // ── Rename/To ────────────────────────────────────────────────
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    let (created, renamed) = process_rename_to(
                        event.paths,
                        &mut completed_downloads,
                        &mut pending_rename_from,
                    );

                    if !created.is_empty() {
                        let _ = tx
                            .send(HomeMessage::WatcherEvent(WatcherEvent::Created(created)).into())
                            .await;
                    }
                    if !renamed.is_empty() {
                        let _ = tx
                            .send(HomeMessage::WatcherEvent(WatcherEvent::Renamed(renamed)).into())
                            .await;
                    }
                }

                // ── Rename/Both (Linux inotify) ──────────────────────────────
                // Carries [from, to] in a single event. Mirror what separate
                // From + To events would do, applying the full download heuristic.
                EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                    if let [from, to] = event.paths.as_slice() {
                        // Simulate the From half:
                        if pending_downloads.remove(from) {
                            completed_downloads.push_back(from.clone());
                        } else {
                            pending_rename_from.push(from.clone());
                        }

                        // Simulate the To half:
                        let (created, renamed) = process_rename_to(
                            vec![to.clone()],
                            &mut completed_downloads,
                            &mut pending_rename_from,
                        );

                        if !created.is_empty() {
                            let _ = tx
                                .send(
                                    HomeMessage::WatcherEvent(WatcherEvent::Created(created))
                                        .into(),
                                )
                                .await;
                        }
                        if !renamed.is_empty() {
                            let _ = tx
                                .send(
                                    HomeMessage::WatcherEvent(WatcherEvent::Renamed(renamed))
                                        .into(),
                                )
                                .await;
                        }
                    }
                }

                // ── Remove ───────────────────────────────────────────────────
                EventKind::Remove(_) => {
                    let mut removed = Vec::new();

                    for path in event.paths {
                        if reserved_names.remove(&path) {
                            // Chrome placeholder cleanup — silent.
                            continue;
                        }
                        if pending_downloads.remove(&path) {
                            // Cancelled in-flight download — clean up, no event.
                            continue;
                        }
                        removed.push(path);
                    }

                    if !removed.is_empty() {
                        let _ = tx
                            .send(HomeMessage::WatcherEvent(WatcherEvent::Removed(removed)).into())
                            .await;
                    }
                }

                // ── Modify/Write, Modify/Metadata, etc. — ignored ───────────
                _ => {}
            }
        }
    })
}

/// Classifies each RenameTo path as either a completed download (Created),
/// a rename pair (Renamed), or a move-in from outside (Created unconfirmed).
///
/// Uses positional FIFO matching for downloads — correct because the OS
/// always delivers From -> To pairs in causal order.
fn process_rename_to(
    to_paths: Vec<PathBuf>,
    completed_downloads: &mut VecDeque<PathBuf>,
    pending_rename_from: &mut Vec<PathBuf>,
) -> (Vec<(PathBuf, bool)>, Vec<(PathBuf, PathBuf)>) {
    let mut created = Vec::new();
    let mut renamed = Vec::new();

    for to in to_paths {
        if completed_downloads.pop_front().is_some() {
            // A graduated download temp was waiting — this is its final name.
            created.push((to, true));
        } else if let Some(from) = pending_rename_from.drain(..1).next() {
            // A real file rename pair.
            renamed.push((from, to));
        } else {
            // No pending From of any kind — file moved in from outside.
            created.push((to, false));
        }
    }

    (created, renamed)
}

/// Returns true if this path looks like a browser download temp file.
/// Covers all known schemes:
///   .crdownload  — Chromium family (Chrome, Brave, Edge, Opera, Vivaldi)
///                  both "filename.ext.crdownload" and "Unconfirmed NNNNNN.crdownload"
///   .part        — Firefox ("filename.ext.part")
///   .tmp         — Chromium on Windows (UUID.tmp initial phase)
///                  also used by some download managers
fn is_download_temp(path: &Path) -> bool {
    DOWNLOAD_TEMP_EXTENSIONS.iter().any(|ext| path.extension().is_some_and(|e| e == *ext))
}

fn sort_directory(
    root: PathBuf,
    registry: Arc<IndexMap<PathBuf, ExplorerNode>>,
    rules: Vec<Rule>,
    overwrite: bool,
) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, move |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let mut visited = IndexMap::with_capacity(registry.len());
        let mut new_paths = HashSet::with_capacity(registry.len());

        for (_, node) in registry.iter() {
            let mut moved: Option<(PathBuf, PathBuf)> = None;
            for rule in &rules {
                if rule.applies_to(&node.path)
                    && !node.unidentified
                    && !visited.contains_key(&node.path)
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
                        tracing::error!(
                            "failed to create destination directory for {:?}",
                            dest_path
                        );
                        continue;
                    };
                    let Ok(_) = smol::fs::rename(&node.path, &dest_path).await else {
                        tracing::error!("failed to rename file {:?} to {:?}", node.path, dest_path);
                        continue;
                    };

                    moved = Some((node.path.clone(), dest_path.clone()));
                    visited.insert(node.path.clone(), dest_path.clone());
                    new_paths.insert(dest_path);
                    break;
                }
            }
            if let Some(moved) = moved {
                let _ = tx.send(HomeMessage::SortingProgress(moved).into()).await;
            }
        }
        let _ = tx.send(HomeMessage::SortingComplete.into()).await;
    })
}

fn purge_empty_dirs(
    registry: Arc<IndexMap<PathBuf, ExplorerNode>>,
) -> impl futures::Stream<Item = GlobalMessage> {
    stream::channel(100, move |mut tx: futures::channel::mpsc::Sender<GlobalMessage>| async move {
        let mut dirs: Vec<&PathBuf> =
            registry.iter().filter(|(_, node)| node.is_dir).map(|(path, _)| path).collect();
        dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

        let mut removed: HashSet<&PathBuf> = HashSet::new();
        let mut it = dirs.into_iter();

        while let Some(path) = it.next_back() {
            let Some(node) = registry.get(path) else { continue };
            let all_children_removed = node.children.iter().all(|child| removed.contains(child));

            if node.children.is_empty() || all_children_removed {
                let _ = smol::fs::remove_dir(path).await;
                removed.insert(path);
                let _ = tx.send(HomeMessage::PurgeProgress(path.clone()).into()).await;
            }
        }
        let _ = tx.send(HomeMessage::PurgeComplete.into()).await;
    })
}
