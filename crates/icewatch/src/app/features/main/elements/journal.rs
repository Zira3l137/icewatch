use std::path::Path;

use chrono::Duration;
use chrono::Local;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::alignment::Vertical;
use iced::theme::palette::Extended;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::rich_text;
use iced::widget::row;
use iced::widget::rule;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text::Span;
use iced::widget::text_input;
use icewatch_utils::locale::Locale;

use crate::app::features::COL_PADDING;
use crate::app::features::COL_SPACING;
use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ICON_SIZE;
use crate::app::features::ROW_PADDING;
use crate::app::features::ROW_SPACING;
use crate::app::features::SCROLLBAR_SPACING;
use crate::app::features::SEPARATOR_SIZE;
use crate::app::features::main::Context;
use crate::app::features::main::GlobalMessage;
use crate::app::features::main::JournalMessage;
use crate::app::main::data::JournalEntrySection;
use crate::journal::Action;
use crate::journal::ActionType;
use crate::journal::Entry;
use crate::journal::Journal;

pub(crate) fn journal_entry_section<'a>(
    section: JournalEntrySection,
    journal: &Journal,
    locale: &'a Locale,
    palette: &'a Extended,
) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let (section_name, entries) = match section {
        JournalEntrySection::Today => (
            local("today_entries"),
            journal.entries_between(Local::now() - Duration::days(1), Local::now()),
        ),
        JournalEntrySection::Yesterday => (
            local("yesterday_entries"),
            journal.entries_between(
                Local::now() - Duration::days(2),
                Local::now() - Duration::days(1),
            ),
        ),
        JournalEntrySection::All => {
            (local("all_entries"), journal.entries_before(Local::now() - Duration::days(2)))
        }
    };

    column([
        section_title(section_name),
        container(
            entries
                .iter()
                .rev()
                .fold(column![].padding(COL_PADDING).spacing(COL_SPACING), |col, entry| {
                    col.push(journal_entry(*entry, locale, palette))
                }),
        )
        .width(Length::Fill)
        .style(container::bordered_box)
        .padding(CONTAINER_PADDING)
        .into(),
    ])
    .into()
}

fn journal_entry<'a>(
    entry: &Entry,
    locale: &'a Locale,
    palette: &'a Extended,
) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let (action, action_color, action_text, action_type): (String, Color, String, ActionType) =
        match &entry.action {
            Action::Moved { source, destination } => (
                format!("{}", local("journal_entry_moved")),
                palette.warning.base.color,
                format!("{} -> {}", short_path(source, 1), short_path(destination, 2),),
                entry.action_type.clone(),
            ),
            Action::Renamed { source, destination } => (
                format!("{}", local("journal_entry_renamed")),
                palette.warning.base.color,
                format!("{} -> {}", short_path(source, 1), short_path(destination, 2),),
                entry.action_type.clone(),
            ),
            Action::Removed(path) => (
                format!("{}", local("journal_entry_removed")),
                palette.danger.base.color,
                format!("{}", path.display()),
                entry.action_type.clone(),
            ),
            Action::Downloaded(path) => (
                format!("{}", local("journal_entry_downloaded")),
                palette.success.base.color,
                format!("{}", path.display()),
                entry.action_type.clone(),
            ),
        };

    let entry_action_text: Element<'a, GlobalMessage> =
        rich_text![Span::<()>::new(action).color(action_color)].size(ICON_SIZE).into();
    let entry_contents: Element<'a, GlobalMessage> = text(action_text).into();
    let entry_time: Element<'a, GlobalMessage> =
        text(entry.time.format("%H:%M:%S %Y-%m-%d").to_string())
            .size(ICON_SIZE)
            .color(palette.primary.base.color)
            .into();
    let entry_action_type: Element<'a, GlobalMessage> = text(match action_type {
        ActionType::Automatic => "A",
        ActionType::Manual => "U",
    })
    .size(ICON_SIZE)
    .color(palette.secondary.base.color)
    .into();

    container(
        row([
            entry_action_text,
            scrollable(entry_contents)
                .width(Length::FillPortion(6))
                .horizontal()
                .spacing(SCROLLBAR_SPACING / 2.0)
                .into(),
            space::horizontal().into(),
            entry_time,
            entry_action_type,
        ])
        .align_y(Vertical::Center)
        .spacing(ROW_SPACING)
        .width(Length::Fill),
    )
    .style(container::bordered_box)
    .padding(CONTAINER_PADDING)
    .into()
}

fn section_title<'a>(section_name: &'a str) -> Element<'a, GlobalMessage> {
    column([text(section_name).size(ICON_SIZE).into(), rule::horizontal(SEPARATOR_SIZE).into()])
        .padding(COL_PADDING)
        .spacing(COL_SPACING)
        .into()
}

fn short_path(path: &Path, max_depth: usize) -> String {
    path.components()
        .rev()
        .take(max_depth)
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn filter_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let search_bar: Element<'a, GlobalMessage> =
        text_input(local("search_bar"), &ctx.feature_state.journal_filter)
            .on_input(|i| JournalMessage::JournalFilterInput(i).into())
            .on_paste(|p| JournalMessage::JournalFilterInput(p).into())
            .into();

    let abort_btn: Element<'a, GlobalMessage> =
        button(local("abort_btn")).on_press(JournalMessage::JournalFilterClear.into()).into();

    container(row([search_bar, abort_btn]).padding(ROW_PADDING).spacing(ROW_SPACING))
        .height(Length::Shrink)
        .width(Length::Fill)
        .align_y(Vertical::Top)
        .padding(CONTAINER_PADDING)
        .style(container::bordered_box)
        .into()
}
