macro_rules! register_features {
    (
        $( $module:ident::$feature:ident ),+ $(,)?
    ) => {

        #[derive(Debug, Clone)]
        pub(crate) enum FeatureMessage {
            $(
                $feature($crate::app::features::$module::Message),
            )+
        }

        #[derive(Debug, Default)]
        pub(crate) struct FeaturesState {
            $(
                pub $module :$crate::app::features::$module::State,
            )+
        }

        pub(crate) fn route_feature_update(
            app: &mut $crate::app::App,
            msg: FeatureMessage
        ) -> iced::Task<$crate::app::message::Message> {
            match msg {
                $(
                    FeatureMessage::$feature(fmsg) => $crate::app::features::$module::update(
                        fmsg,
                        $crate::app::features::$module::ContextMut::new(app)
                    ),
                )+
            }
        }

        pub(crate) fn initialize_features(app: &mut $crate::app::App) {
            $(
                $crate::app::features::$module::init($crate::app::features::$module::ContextMut::new(app));
            )+
        }
    };
}

macro_rules! register_windows {
    (
        $(
            $window:ident {
                settings: $settings:expr,
                view_handler: $view:path,
                input_handler: $input:path,
                context: $context:path
            }
        ),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) enum Window {
            $(
                $window,
            )+
        }

        impl Window {
            pub(crate) fn title(&self) -> &str {
                match self {
                    $(
                        Window::$window => stringify!($window),
                    )+
                }
            }

            pub(crate) fn settings(&self) -> iced::window::Settings {
                match self {
                    $(
                        Window::$window => $settings,
                    )+
                }
            }

            pub(crate) fn input(&self, input_event: &$crate::app::message::InputEvent) -> iced::Task<$crate::app::message::Message> {
                match self {
                    $(
                        Window::$window => $input(input_event),
                    )+
                }
            }

            pub(crate) fn view<'a>(
                &self,
                app: &'a $crate::app::App,
                wnd_id: iced::window::Id
            ) -> iced::Element<'a, $crate::app::message::Message> {
                match self {
                    $(
                        Window::$window => $view($context(app), wnd_id),
                    )+
                }
            }
        }
    };
}

pub(crate) use {register_features, register_windows};
