use iced::Element;
use iced::Length;
use iced::Theme;
use iced::widget::column;
use iced::widget::container;
use iced::widget::rich_text;
use iced::widget::row;
use iced::widget::rule;
use iced::widget::space;
use iced::widget::text::Span;
use iced::widget::text::Text;
use icewatch_utils::locale::Locale;

use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ICON_SIZE;
use crate::app::features::PROGRESS_FRAME_COUNT;
use crate::app::features::PROGRESS_INTERVAL_MS;
use crate::app::features::ROW_PADDING;
use crate::app::features::ROW_SPACING;
use crate::app::features::SEPARATOR_SIZE;
use crate::app::features::main::Context;
use crate::app::features::main::GlobalMessage;

pub(crate) fn dashboard<'a>(
    ctx: Context<'a>,
    locale: &'a Locale,
    theme: &'a Theme,
) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let last_redraw = ctx.last_redraw.clone();
    let progress = last_redraw.as_millis() / PROGRESS_INTERVAL_MS % PROGRESS_FRAME_COUNT as u128;
    let palette = theme.extended_palette();
    let last_sorted: (String, String) = ctx
        .feature_state
        .last_sorted_file
        .clone()
        .map(|(old, new)| (old.to_string_lossy().into_owned(), new.to_string_lossy().into_owned()))
        .unwrap_or_default();

    let current_date = rich_text([Span::<()>::new(ctx.date.format("%d-%m-%Y").to_string())
        .size(ICON_SIZE * 2)
        .color(palette.primary.base.color)]);

    let status_style =
        if *ctx.watch_status { palette.success.base } else { palette.danger.base }.color;
    let status_text = if *ctx.watch_status { local("watching") } else { local("paused") };
    let current_status =
        rich_text([Span::<()>::new(status_text).size(ICON_SIZE * 2).color(status_style)]);

    let dwnld_count = rich_text([
        Span::<()>::new(local("dwnld_icon")).size(ICON_SIZE).color(palette.primary.base.color),
        Span::new(ctx.feature_state.downloaded_count),
    ]);

    let sorted_count = rich_text([
        Span::<()>::new(local("sorted_icon")).size(ICON_SIZE).color(palette.primary.base.color),
        Span::new(ctx.feature_state.moved.len()),
    ]);

    let last_file_info = rich_text([
        Span::<()>::new(format!("{} ", last_sorted.0))
            .size(ICON_SIZE)
            .color(palette.secondary.base.color),
        Span::new(" ").size(ICON_SIZE),
        Span::new(format!(" {} ", last_sorted.1)).size(ICON_SIZE).color(palette.primary.base.color),
    ]);

    let h_separator = || rule::horizontal(SEPARATOR_SIZE).into();
    // let w_separator = || rule::vertical(SEPARATOR_SIZE).into();

    let current_indicator_symbol = local("loading_icons").chars().nth(progress as _).unwrap_or('⠋');
    let loading_indicator: Element<'a, GlobalMessage> = if ctx.feature_state.is_loading {
        Text::new(format!(" {current_indicator_symbol} "))
            .color(palette.primary.base.color)
            .size(ICON_SIZE * 2)
            .into()
    } else {
        space().into()
    };

    let status_row = row([
        current_status.into(),
        loading_indicator,
        space::horizontal().into(),
        current_date.into(),
    ])
    .width(Length::Fill)
    .padding(ROW_PADDING)
    .spacing(ROW_SPACING)
    .into();

    let info_row = row([
        dwnld_count.into(),
        sorted_count.into(),
        space::horizontal().into(),
        last_file_info.into(),
    ])
    .width(Length::Fill)
    .padding(ROW_PADDING)
    .spacing(ROW_SPACING)
    .into();

    container(
        column([status_row, h_separator(), info_row]).padding(ROW_PADDING).spacing(ROW_SPACING),
    )
    .align_top(Length::Shrink)
    .padding(CONTAINER_PADDING)
    .style(container::bordered_box)
    .into()
}
