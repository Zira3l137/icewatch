use super::Context;

use super::super::{CONTAINER_PADDING, ROW_PADDING, ROW_SPACING};
use super::{GlobalMessage, Message};

use iced::widget::text_input;
use iced::{
    Element, Length,
    alignment::Vertical,
    widget::{button, column, container, row, space},
};
use icewatch_utils::locale::Locale;

pub(crate) fn toolbar<'a>(locale: &'a Locale, ctx: Context<'a>) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);

    let toggle_style = if *ctx.watch_status { button::danger } else { button::success };
    let toggle_icon = if *ctx.watch_status { local("pause_btn") } else { local("resume_btn") };
    let toggle_btn = button(toggle_icon).on_press(Message::ToggleWatch.into()).style(toggle_style);
    let update_btn = button(local("update_btn")).on_press(Message::RunFullPipeline.into());

    let rules_btn = button(local("rules_btn")).on_press(Message::OpenRules.into());
    let settings_btn = button(local("settings_btn")).on_press(Message::OpenSettings.into());
    let chroot_btn = button(local("chroot_btn")).on_press_with(move || {
        let new_root = rfd::FileDialog::new()
            .set_title(local("pick_root"))
            .set_file_name(dirs::download_dir().unwrap_or_default().to_string_lossy())
            .pick_folder();
        Message::ChangeRoot(new_root).into()
    });

    let controls_row: Element<'a, GlobalMessage> = row([
        toggle_btn.into(),
        update_btn.into(),
        space::horizontal().into(),
        rules_btn.into(),
        chroot_btn.into(),
        settings_btn.into(),
    ])
    .padding(ROW_PADDING)
    .spacing(ROW_SPACING)
    .into();

    let search_bar: Element<'a, GlobalMessage> =
        text_input(local("search_bar"), &ctx.feature_state.search_query)
            .on_input(|i| Message::SearchBarInput(i).into())
            .on_paste(|p| Message::SearchBarInput(p).into())
            .on_submit(Message::SearchBarSubmit.into())
            .into();

    let submit_btn: Element<'a, GlobalMessage> =
        button(local("submit_btn")).on_press(Message::SearchBarSubmit.into()).into();

    let abort_btn: Element<'a, GlobalMessage> =
        button(local("abort_btn")).on_press(Message::SearchClear.into()).into();

    let search_row: Element<'a, GlobalMessage> =
        row([search_bar, submit_btn, abort_btn]).padding(ROW_PADDING).spacing(ROW_SPACING).into();

    container(column![controls_row, search_row])
        .height(Length::Shrink)
        .width(Length::Fill)
        .align_y(Vertical::Top)
        .padding(CONTAINER_PADDING)
        .style(container::bordered_box)
        .into()
}
