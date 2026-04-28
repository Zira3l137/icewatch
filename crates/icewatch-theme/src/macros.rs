macro_rules! register_themes {
    (
        $(
            $theme_name:ident => $palette:expr
        ),+ $(,)?
    ) => {
        pub fn registered_themes() -> Vec<iced::Theme> {
            [
                $(iced::Theme::custom(stringify!($theme_name), $palette),)+
            ].to_vec()
        }
    };
}

pub(crate) use register_themes;
