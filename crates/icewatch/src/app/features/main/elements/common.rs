use iced::Element;
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::button;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use icewatch_utils::locale::Locale;

use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ROW_PADDING;
use crate::app::features::ROW_SPACING;
use crate::app::features::main::Message;
use crate::app::message::Message as GlobalMessage;

pub(crate) fn return_panel<'a>(locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let return_btn = button(text(local("return_btn")).center())
        .width(Length::Fill)
        .on_press(Message::Return.into());
    container(row![return_btn].spacing(ROW_SPACING).padding(ROW_PADDING))
        .height(Length::Shrink)
        .width(Length::Fill)
        .align_y(Vertical::Top)
        .padding(CONTAINER_PADDING)
        .style(container::bordered_box)
        .into()
}
