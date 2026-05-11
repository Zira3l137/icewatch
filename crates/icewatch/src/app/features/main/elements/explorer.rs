use std::path::PathBuf;
use std::time::Duration;

use chrono::DateTime;
use chrono::Local;
use iced::Background;
use iced::Color;
use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::Vertical;
use iced::theme::palette::Extended;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::mouse_area;
use iced::widget::rich_text;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text::Span;
use iced::widget::tooltip;
use icewatch_utils::locale::Locale;

use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ICON_SIZE;
use crate::app::features::SCROLLBAR_SPACING;
use crate::app::features::TOOLTIP_DELAY_MS;
use crate::app::features::main::Context;
use crate::app::features::main::GlobalMessage;
use crate::app::features::main::HomeMessage;

#[derive(Debug, Clone)]
pub(crate) struct ExplorerNode {
    pub unidentified: bool,
    pub created: DateTime<Local>,
    pub size_bytes: u64,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<PathBuf>,
}

pub(crate) fn explorer<'a>(
    ctx: Context<'a>,
    locale: &'a Locale,
    theme: &'a Theme,
) -> Element<'a, GlobalMessage> {
    let palette = theme.extended_palette();

    let root_node = ExplorerNode {
        size_bytes: 0,
        created: Local::now(),
        unidentified: false,
        path: ctx.root_directory.into(),
        is_dir: true,
        expanded: true,
        children: ctx
            .feature_state
            .root_registry
            .keys()
            .filter(|p| p.parent() == Some(ctx.root_directory))
            .cloned()
            .collect(),
    };

    let mut nodes = Vec::with_capacity(ctx.feature_state.root_registry.len() + 1);

    create_nodes(&mut nodes, &root_node, 0, ctx.clone(), palette, locale);
    let node_col = nodes.into_iter().fold(column![], |acc, node| acc.push(node));

    scrollable(
        container(mouse_area(node_col).on_exit(HomeMessage::ClearFocus.into()))
            .height(Length::Shrink)
            .width(Length::Fill)
            .align_y(Vertical::Top)
            .padding(CONTAINER_PADDING)
            .style(container::bordered_box),
    )
    .spacing(SCROLLBAR_SPACING)
    .auto_scroll(true)
    .into()
}

pub(crate) fn create_nodes<'a>(
    acc: &mut Vec<Element<'a, GlobalMessage>>,
    root: &ExplorerNode,
    depth: usize,
    ctx: Context<'a>,
    palette: &'a Extended,
    locale: &'a Locale,
) {
    if !root.expanded {
        return;
    }

    let registry = if !ctx.feature_state.search_requested {
        &ctx.feature_state.root_registry
    } else {
        &ctx.feature_state.search_results
    };

    let mut children = root.children.iter().collect::<Vec<_>>();
    children.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()).then(a.cmp(b)));

    children.iter().for_each(|p| {
        let Some(node) = registry.get(*p) else { return };
        let focused = ctx.feature_state.focused_node.as_deref() == Some(node.path.as_path());
        acc.push(node_widget(node, depth, focused, palette, locale));
        if node.is_dir && node.expanded {
            create_nodes(acc, node, depth + 1, ctx.clone(), palette, locale);
        }
    });
}

pub(crate) fn node_widget<'a>(
    node: &'a ExplorerNode,
    depth: usize,
    focused: bool,
    palette: &'a Extended,
    locale: &'a Locale,
) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);

    let main_content = node_button(node, depth, focused, palette, &local);
    let tooltip_content = node_tooltip(node, palette, &local);

    tooltip(main_content, tooltip_content, tooltip::Position::FollowCursor)
        .delay(Duration::from_millis(TOOLTIP_DELAY_MS))
        .into()
}

fn node_button<'a>(
    node: &'a ExplorerNode,
    depth: usize,
    focused: bool,
    palette: &'a Extended,
    local: &impl Fn(&str) -> &'a str,
) -> Element<'a, GlobalMessage> {
    let btn = node_label_button(node, focused, palette, local);

    mouse_area(
        row([
            space::horizontal().width(depth as f32 * 16.0).into(),
            text(if depth > 0 { "   └─" } else { "" }).into(),
            btn,
        ])
        .align_y(Vertical::Center),
    )
    .on_right_release(HomeMessage::ToggleContextMenu(node.path.clone()).into())
    .on_enter(HomeMessage::FocusNode(node.path.clone()).into())
    .into()
}

fn node_label_button<'a>(
    node: &'a ExplorerNode,
    focused: bool,
    palette: &'a Extended,
    local: &impl Fn(&str) -> &'a str,
) -> Element<'a, GlobalMessage> {
    let fg_color = node_icon_fg_color(node, focused, palette);

    let toggle_icon = node_toggle_icon(node, local);
    let file_name =
        node.path.file_name().and_then(|n| n.to_str()).unwrap_or_else(|| local("file_name_fail"));

    let label: Element<'a, GlobalMessage> = rich_text([
        Span::<()>::new(toggle_icon).size(ICON_SIZE).color(fg_color),
        Span::new(file_name),
    ])
    .align_y(Vertical::Center)
    .into();

    let indicator: Element<'a, GlobalMessage> = if node.unidentified {
        text(local("unidentified"))
            .color(palette.warning.base.color)
            .align_y(Vertical::Center)
            .into()
    } else {
        space().into()
    };

    let container_color = if focused { palette.primary.base.color } else { Color::TRANSPARENT };
    let container_style = move |theme: &Theme| {
        container::bordered_box(theme).background(Background::from(container_color))
    };

    let text_row: Element<'a, GlobalMessage> =
        row([scrollable(label).horizontal().into(), space::horizontal().into(), indicator])
            .align_y(Vertical::Center)
            .into();

    button(container(text_row).style(container_style).padding(2.0))
        .on_press(HomeMessage::ExpandNode(node.path.clone()).into())
        .width(Length::Fill)
        .style(button::text)
        .into()
}

fn node_icon_fg_color<'a>(node: &'a ExplorerNode, focused: bool, palette: &'a Extended) -> Color {
    if node.is_dir {
        if focused { palette.warning.base.text } else { palette.warning.base.color }
    } else {
        if focused { palette.primary.base.text } else { palette.primary.base.color }
    }
}

fn node_toggle_icon<'a>(node: &'a ExplorerNode, local: &impl Fn(&str) -> &'a str) -> &'a str {
    if node.is_dir {
        if node.expanded { local("folder_open_icon") } else { local("folder_icon") }
    } else {
        local("file_icon")
    }
}

fn node_tooltip<'a>(
    node: &'a ExplorerNode,
    palette: &'a Extended,
    local: &impl Fn(&str) -> &'a str,
) -> Element<'a, GlobalMessage> {
    let unidentified_label = local("unidentified");

    let value_span = |value: Option<String>| -> Span<'a> {
        match value {
            Some(v) => Span::new(v).color(palette.primary.base.color),
            None => Span::new(unidentified_label),
        }
    };

    let node_size_bytes = node.size_bytes;
    let node_size_mb = ((node_size_bytes as f64 / 1024.0) / 1024.0).round();
    let size_str = if node_size_mb < 0.0001 {
        format!("{}", node_size_bytes)
    } else {
        format!("{} ({:.4} MB)", node_size_bytes, node_size_mb)
    };
    let date_val = (!node.unidentified).then(|| node.created.format("%d-%m-%Y %H:%M").to_string());
    let size_val = (!node.unidentified).then_some(size_str);

    container(column([
        rich_text([Span::<()>::new(local("file_date")), value_span(date_val)]).into(),
        rich_text([Span::<()>::new(local("file_size")), value_span(size_val)]).into(),
    ]))
    .padding(CONTAINER_PADDING)
    .style(container::bordered_box)
    .into()
}
