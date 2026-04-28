use super::{CONTAINER_PADDING, Context, GlobalMessage, Message};

use iced::{
    Element, Vector,
    widget::{button, column, container, float},
};
use icewatch_utils::locale::Locale;

pub(crate) fn context_menu<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let btn_style = button::text;
    let open_btn = button(local("open_btn")).on_press(Message::OpenNode.into()).style(btn_style);
    let show_btn = button(local("show_btn")).on_press(Message::ShowNode.into()).style(btn_style);

    float(
        container(column![open_btn, show_btn])
            .padding(CONTAINER_PADDING)
            .style(container::bordered_box),
    )
    .scale(1.0)
    .translate(|_, _| {
        let mouse_pos = ctx.feature_state.last_mouse_position;
        Vector::new(mouse_pos.x, mouse_pos.y)
    })
    .into()
}
