use iced::Color;
use iced::Element;
use iced::Length;
use iced::alignment::Vertical;
use iced::mouse;
use iced::theme::palette::Extended;
use iced::widget::button;
use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::container;
use iced::widget::mouse_area;
use iced::widget::rich_text;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text::Span;
use iced::widget::text_input;
use icewatch_utils::locale::Locale;

use crate::app::features::COL_PADDING;
use crate::app::features::COL_SPACING;
use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ROW_PADDING;
use crate::app::features::ROW_SPACING;
use crate::app::features::main::Context;
use crate::app::features::main::Criterion;
use crate::app::features::main::CriterionKind;
use crate::app::features::main::RulesMessage;
use crate::app::message::Message as GlobalMessage;
use crate::rules::Rule;

pub(crate) fn rules_panel<'a>(
    ctx: Context<'a>,
    locale: &'a Locale,
    palette: &'a Extended,
) -> Element<'a, GlobalMessage> {
    let no_rules = ctx.sorting_rules.is_empty();
    if no_rules {
        space().into()
    } else {
        container(list_box(ctx.clone(), locale, palette))
            .height(Length::Shrink)
            .width(Length::Fill)
            .align_y(Vertical::Top)
            .padding(CONTAINER_PADDING)
            .style(container::bordered_box)
            .into()
    }
}

pub(crate) fn control_panel<'a>(
    ctx: Context<'a>,
    locale: &'a Locale,
) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    container(
        row![
            button(text(local("add_rule_btn")).center())
                .style(button::success)
                .on_press(RulesMessage::AddRule.into())
                .width(Length::Fill),
            button(text(local("remove_rule_btn")).center())
                .style(button::danger)
                .on_press(RulesMessage::RemoveRule(ctx.feature_state.focused_rule).into())
                .width(Length::Fill),
        ]
        .spacing(COL_SPACING)
        .padding(COL_PADDING),
    )
    .height(Length::Shrink)
    .width(Length::Fill)
    .align_y(Vertical::Top)
    .padding(CONTAINER_PADDING)
    .style(container::bordered_box)
    .into()
}

pub(crate) fn edit_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    if !ctx.feature_state.rule_mode {
        return space().into();
    }

    let local = |key: &str| locale.get_string("main", key);
    let fs = ctx.feature_state;

    let sorting_selector = combo_box(
        &fs.sorting_state,
        local("sorting_placeholder"),
        Some(&fs.active_criterion),
        |s| RulesMessage::SetCriterion(s).into(),
    );

    let input_section: Element<'a, GlobalMessage> = match &fs.active_criterion {
        Criterion::ByExtension => {
            text_input(local("extension"), &fs.extension_input.clone().unwrap_or_default())
                .on_input(|i| RulesMessage::ExtensionInput(i).into())
                .into()
        }
        Criterion::ByName => column![
            text_input(local("starts_with"), &fs.starts_with_input.clone().unwrap_or_default())
                .on_input(|i| RulesMessage::StartsWithInput(i).into()),
            text_input(local("ends_with"), &fs.ends_with_input.clone().unwrap_or_default())
                .on_input(|i| RulesMessage::EndsWithInput(i).into()),
            text_input(local("contains"), &fs.contains_input.clone().unwrap_or_default())
                .on_input(|i| RulesMessage::ContainsInput(i).into()),
        ]
        .padding(COL_PADDING)
        .spacing(COL_SPACING)
        .into(),
    };

    let destination_input =
        text_input(local("destination"), &fs.destination_input.clone().unwrap_or_default())
            .on_input(|i| RulesMessage::DestinationInput(i).into());

    let controls = row![
        button(text(local("apply_btn")))
            .width(Length::Fill)
            .style(button::success)
            .on_press(RulesMessage::ApplyRuleEdit(fs.focused_rule).into()),
        button(text(local("cancel_btn")))
            .width(Length::Fill)
            .style(button::danger)
            .on_press(RulesMessage::CancelEdit.into()),
    ]
    .padding(ROW_PADDING)
    .spacing(ROW_SPACING);

    container(
        column![
            sorting_selector,
            container(column![input_section, destination_input, controls])
                .style(container::bordered_box)
                .padding(CONTAINER_PADDING)
        ]
        .padding(COL_PADDING)
        .spacing(COL_SPACING),
    )
    .height(Length::Shrink)
    .width(Length::Fill)
    .align_y(Vertical::Top)
    .padding(CONTAINER_PADDING)
    .style(container::bordered_box)
    .into()
}

fn list_box<'a>(
    ctx: Context<'a>,
    locale: &'a Locale,
    palette: &'a iced::theme::palette::Extended,
) -> Element<'a, GlobalMessage> {
    let items: Vec<Element<'a, GlobalMessage>> = ctx
        .sorting_rules
        .iter()
        .enumerate()
        .map(|(rule_idx, rule)| {
            let focused = ctx.feature_state.focused_rule == Some(rule_idx);
            let bg = if focused { palette.primary.base.color } else { Color::TRANSPARENT };
            let interaction = if ctx.feature_state.rule_mode {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::Pointer
            };
            let rule_idx = (!ctx.feature_state.rule_mode).then_some(rule_idx);

            mouse_area(
                container(rule_text(rule, focused, locale, palette))
                    .padding(CONTAINER_PADDING)
                    .style(move |theme| container::bordered_box(theme).background(bg)),
            )
            .interaction(interaction)
            .on_press(RulesMessage::FocusRule(rule_idx).into())
            .on_double_click(RulesMessage::EditRule(ctx.feature_state.focused_rule).into())
            .into()
        })
        .collect();

    column(items).padding(COL_PADDING).spacing(COL_SPACING).into()
}

fn rule_text<'a>(
    rule: &Rule,
    focused: bool,
    locale: &'a Locale,
    palette: &'a iced::theme::palette::Extended,
) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let primary_color =
        if focused { palette.primary.base.text } else { palette.primary.base.color };
    let secondary_color =
        if focused { palette.success.base.text } else { palette.success.base.color };
    rich_text(match &rule.criterion {
        CriterionKind::ByExtension(crit) => vec![
            Span::<()>::new(format!("{}: ", local("extension"))),
            Span::new(format!("\"{}\"", &crit.extensions.join(", "))).color(primary_color),
            Span::new(" → "),
            Span::new(rule.destination.to_string_lossy().into_owned()).color(secondary_color),
        ],
        CriterionKind::ByName(crit) => vec![
            Span::new("Prefix: "),
            Span::new(crit.starts_with.clone().unwrap_or_default()).color(primary_color),
            Span::new(" - Suffix: "),
            Span::new(crit.ends_with.clone().unwrap_or_default()).color(primary_color),
            Span::new(" - Contains: "),
            Span::new(crit.contains.clone().unwrap_or_default()).color(primary_color),
            Span::new(" → "),
            Span::new(rule.destination.to_string_lossy().into_owned()).color(secondary_color),
        ],
    })
    .width(Length::Fill)
    .into()
}
