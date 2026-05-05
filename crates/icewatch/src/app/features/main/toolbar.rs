use iced::Element;
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text_input;
use icewatch_utils::locale::Locale;

use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ROW_PADDING;
use crate::app::features::ROW_SPACING;
use crate::app::features::main::Context;
use crate::app::features::main::GlobalMessage;
use crate::app::features::main::message::HomeMessage;
use crate::journal::ActionType;

pub(crate) fn toolbar<'a>(locale: &'a Locale, ctx: Context<'a>) -> Element<'a, GlobalMessage> {
    let controls_panel = controls_panel(ctx.clone(), locale);
    let search_panel = search_panel(ctx.clone(), locale);

    container(column![controls_panel, search_panel])
        .height(Length::Shrink)
        .width(Length::Fill)
        .align_y(Vertical::Top)
        .padding(CONTAINER_PADDING)
        .style(container::bordered_box)
        .into()
}

fn search_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let search_bar: Element<'a, GlobalMessage> =
        text_input(local("search_bar"), &ctx.feature_state.search_query)
            .on_input(|i| HomeMessage::SearchBarInput(i).into())
            .on_paste(|p| HomeMessage::SearchBarInput(p).into())
            .on_submit(HomeMessage::SearchBarSubmit.into())
            .into();

    let submit_btn: Element<'a, GlobalMessage> =
        button(local("submit_btn")).on_press(HomeMessage::SearchBarSubmit.into()).into();

    let abort_btn: Element<'a, GlobalMessage> =
        button(local("abort_btn")).on_press(HomeMessage::SearchClear.into()).into();

    row([search_bar, submit_btn, abort_btn]).padding(ROW_PADDING).spacing(ROW_SPACING).into()
}

fn controls_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let toggle_style = if *ctx.watch_status { button::danger } else { button::success };
    let toggle_icon = if *ctx.watch_status { local("pause_btn") } else { local("resume_btn") };
    let toggle_btn =
        button(toggle_icon).on_press(HomeMessage::ToggleWatch.into()).style(toggle_style);
    let update_btn = button(local("update_btn"))
        .on_press(HomeMessage::RunFullPipeline(ActionType::Manual).into());
    let journal_btn = button(local("journal_btn")).on_press(HomeMessage::OpenJournal.into());

    let rules_btn = button(local("rules_btn")).on_press(HomeMessage::OpenRules.into());
    let settings_btn = button(local("settings_btn")).on_press(HomeMessage::OpenSettings.into());
    let chroot_btn = button(local("chroot_btn")).on_press_with(move || {
        let new_root = rfd::FileDialog::new()
            .set_title(local("pick_root"))
            .set_file_name(dirs::download_dir().unwrap_or_default().to_string_lossy())
            .pick_folder();
        HomeMessage::ChangeRoot(new_root).into()
    });

    row([
        toggle_btn.into(),
        update_btn.into(),
        journal_btn.into(),
        space::horizontal().into(),
        rules_btn.into(),
        chroot_btn.into(),
        settings_btn.into(),
    ])
    .padding(ROW_PADDING)
    .spacing(ROW_SPACING)
    .into()
}
