use iced::Element;
use iced::Vector;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::float;
use icewatch_utils::locale::Locale;

use crate::app::features::CONTAINER_PADDING;
use crate::app::features::main::Context;
use crate::app::features::main::GlobalMessage;
use crate::app::features::main::messages::HomeMessage;

pub(crate) fn context_menu<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let btn_style = button::subtle;
    let open_btn =
        button(local("open_btn")).on_press(HomeMessage::OpenNode.into()).style(btn_style);
    let show_btn =
        button(local("show_btn")).on_press(HomeMessage::ShowNode.into()).style(btn_style);

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
