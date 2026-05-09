mod features;
mod message;
mod state;

use std::collections::HashMap;

use iced::Element;
use iced::Subscription;
use iced::Task;
use iced::Theme;
use iced::event;
use iced::theme::Style;
use iced::widget::space;
use iced::window;
use icewatch_persistence::Persistent;
use icewatch_utils::locale::Locale;
use icewatch_utils::locale::get_system_locale;
use message::AppMessage;
use message::Message;
use message::SystemMessage;
use state::AppState;
use state::FeaturesState;
use state::PersistentState;
use state::Window;
use state::initialize_features;
use state::route_feature_update;

use crate::app::features::main;
use crate::app::message::InputEvent;

pub(crate) const STATE_PATH: &str = "state.toml";

#[derive(Default)]
pub(crate) struct App {
    app_state: AppState,
    persistent_state: PersistentState,
    features_state: FeaturesState,
}

impl Persistent for App {
    type State = PersistentState;
}

impl App {
    pub(crate) fn new(
        icon: Option<&window::Icon>,
        locales: &HashMap<String, Locale>,
    ) -> (Self, Task<Message>) {
        tracing::info!("{:-<50}", "");
        tracing::info!("Initializing application");

        let locales = locales.clone();
        let app_state = AppState::new(icon.cloned(), locales);
        let mut persistent_state = <Self as Persistent>::read_state(STATE_PATH).unwrap_or_default();
        if persistent_state.current_locale.is_empty() {
            persistent_state.current_locale = get_system_locale()
        }

        let mut app = Self { app_state, persistent_state, ..Default::default() };
        initialize_features(&mut app);
        (
            app,
            Task::done(Message::App(AppMessage::View(Window::Main)))
                .chain(Task::done(main::HomeMessage::RunFullPipeline.into())),
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Feature(feat_msg) => route_feature_update(self, feat_msg),
            Message::System(sys_msg) => match sys_msg {
                SystemMessage::Exit => {
                    if let Err(e) =
                        <Self as Persistent>::write_state(STATE_PATH, &self.persistent_state)
                    {
                        tracing::error!("Failed to write state: {}", e);
                    };

                    tracing::info!("Exiting application");
                    tracing::info!("{:-<50}", "");
                    iced::exit()
                }

                SystemMessage::Execute(cmd) => {
                    if let Err(err) = cmd.run() {
                        tracing::error!("{err}");
                    } else {
                        tracing::info!("Success: {}", cmd);
                    }
                    Task::none()
                }
            },

            Message::App(wnd_msg) => match wnd_msg {
                AppMessage::Redraw(instant) => {
                    self.app_state.last_redraw =
                        instant.duration_since(self.app_state.startup_instant);
                    Task::none()
                }

                AppMessage::View(target_window) => {
                    let active_window =
                        self.app_state.windows.values().find(|w| *w == &target_window);
                    if active_window.is_some() {
                        return Task::none();
                    }

                    let mut settings = target_window.settings();
                    if settings.icon.is_none() {
                        settings.icon = self.app_state.icon.clone();
                    }

                    let (id, task) = window::open(settings);
                    self.app_state.windows.insert(id, target_window);
                    if Window::Main == target_window {
                        self.app_state.main_window_id = Some(id);
                    }
                    task.discard()
                }

                AppMessage::Hide(target_id) => {
                    let Some(main_id) = self.app_state.main_window_id else {
                        return Task::none();
                    };

                    if self.app_state.windows.remove(&target_id).is_none() {
                        return Task::none();
                    }

                    if self.app_state.windows.is_empty() || target_id == main_id {
                        Task::done(Message::System(SystemMessage::Exit))
                    } else {
                        window::close(target_id)
                    }
                }

                AppMessage::Input(window_id, input) => {
                    let Some(target_window) = self.app_state.windows.get(&window_id) else {
                        return Task::none();
                    };
                    target_window.input(&input)
                }
            },
        }
    }

    pub(crate) fn view<'a>(&'a self, id: window::Id) -> Element<'a, Message> {
        self.app_state
            .windows
            .get(&id)
            .map(|window| window.view(self, id))
            .unwrap_or(space().into())
    }

    pub(crate) fn theme(&self, _: window::Id) -> Theme {
        self.app_state
            .themes
            .get(&self.persistent_state.current_theme)
            .cloned()
            .unwrap_or(Theme::Dark)
    }

    pub(crate) fn style(&self, theme: &Theme) -> Style {
        let palette = theme.palette();
        Style { background_color: palette.background, text_color: palette.text }
    }

    pub(crate) fn title(&self, id: window::Id) -> String {
        let window = self.app_state.windows.get(&id).map(|w| w.title()).unwrap_or("");
        format!("{} - {}", env!("WORKSPACE_NAME"), window)
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let watch_subscription = if self.persistent_state.watch_status {
            Subscription::<Message>::run_with(
                self.persistent_state.root_directory.clone(),
                |root| main::watch_directory_stream(root.clone()),
            )
        } else {
            Subscription::none()
        };

        let frame_subscription = if self.features_state.main.is_loading {
            window::frames().map(|instant| Message::App(AppMessage::Redraw(instant)))
        } else {
            Subscription::none()
        };

        Subscription::batch([
            event::listen_with(|event, _, window_id| match event {
                event::Event::Mouse(mouse_event) => {
                    Some(Message::App(AppMessage::Input(window_id, InputEvent::Mouse(mouse_event))))
                }
                event::Event::Keyboard(keyboard_event) => Some(Message::App(AppMessage::Input(
                    window_id,
                    InputEvent::Keyboard(keyboard_event),
                ))),
                _ => None,
            }),
            frame_subscription,
            window::close_requests().map(|id| Message::App(AppMessage::Hide(id))),
            watch_subscription,
        ])
    }
}
