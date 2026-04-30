use crate::{app::message::Message as GlobalMessage, rules::Rule};
use iced::{
    Color, Element, Length, Theme, mouse,
    theme::palette::Extended,
    widget::{
        button, column, combo_box, container, mouse_area, rich_text, row, space, stack, text,
        text::Span, text_input,
    },
};
use icewatch_utils::locale::Locale;

use super::{
    super::{COL_PADDING, COL_SPACING, CONTAINER_PADDING, ROW_PADDING, ROW_SPACING},
    Context, Criterion, CriterionKind, Message, context_menu, dashboard, explorer, toolbar,
};

#[derive(Debug, Clone, Default)]
pub(crate) enum MainView {
    #[default]
    Home,
    Rules,
    Journal,
}

impl MainView {
    pub(crate) fn view<'a>(
        &self,
        ctx: Context<'a>,
        locale: &'a Locale,
        theme: &'a Theme,
    ) -> Element<'a, GlobalMessage> {
        let palette = theme.extended_palette();
        match self {
            MainView::Home => {
                let toolbar = toolbar::toolbar(&locale, ctx.clone());
                let dashboard = dashboard::dashboard(ctx.clone(), &locale, &theme);
                let explorer = explorer::explorer(ctx.clone(), &locale, &theme);
                let context_menu = if ctx.feature_state.context_menu_visible {
                    context_menu::context_menu(ctx.clone(), &locale)
                } else {
                    space().into()
                };

                let content = container(
                    column![toolbar, dashboard, explorer].spacing(COL_SPACING).padding(COL_PADDING),
                )
                .align_top(Length::Fill)
                .padding(CONTAINER_PADDING)
                .into();

                stack([content, context_menu]).into()
            }
            MainView::Rules => {
                let return_panel: Element<'a, GlobalMessage> = return_panel(locale);
                let control_panel: Element<'a, GlobalMessage> = control_panel(ctx.clone(), locale);
                let edit_panel: Element<'a, GlobalMessage> = edit_panel(ctx.clone(), locale);
                let rules_panel: Element<'a, GlobalMessage> =
                    rules_panel(ctx.clone(), locale, palette);

                container(
                    column![return_panel, control_panel, edit_panel, rules_panel,]
                        .padding(ROW_PADDING)
                        .spacing(ROW_SPACING),
                )
                .align_top(Length::Shrink)
                .padding(CONTAINER_PADDING)
                .style(container::bordered_box)
                .into()
            }
            MainView::Journal => {
                let return_panel: Element<'a, GlobalMessage> = return_panel(locale);
                container(column![return_panel, space()].padding(ROW_PADDING).spacing(ROW_SPACING))
                    .align_top(Length::Shrink)
                    .padding(CONTAINER_PADDING)
                    .style(container::bordered_box)
                    .into()
            }
        }
    }
}

fn return_panel<'a>(locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    let return_btn = button(text(local("return_btn")).center())
        .width(Length::Fill)
        .on_press(Message::ReturnHome.into());
    row![return_btn].spacing(ROW_SPACING).padding(ROW_PADDING).into()
}

fn rules_panel<'a>(
    ctx: Context<'a>,
    locale: &'a Locale,
    palette: &'a Extended,
) -> Element<'a, GlobalMessage> {
    let no_rules = ctx.sorting_rules.is_empty();
    no_rules.then_some(space().into()).unwrap_or(
        container(list_box(ctx.clone(), locale, palette))
            .style(container::bordered_box)
            .padding(CONTAINER_PADDING)
            .into(),
    )
}

fn control_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    let local = |key: &str| locale.get_string("main", key);
    container(
        row![
            button(text(local("add_rule_btn")).center())
                .style(button::success)
                .on_press(Message::AddRule.into())
                .width(Length::Fill),
            button(text(local("edit_rule_btn")).center())
                .style(button::warning)
                .on_press(Message::EditRule(ctx.feature_state.focused_rule).into())
                .width(Length::Fill),
            button(text(local("remove_rule_btn")).center())
                .style(button::danger)
                .on_press(Message::RemoveRule(ctx.feature_state.focused_rule).into())
                .width(Length::Fill),
        ]
        .spacing(COL_SPACING)
        .padding(COL_PADDING),
    )
    .style(container::bordered_box)
    .into()
}

fn edit_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    if !ctx.feature_state.rule_mode {
        return space().into();
    }

    let local = |key: &str| locale.get_string("main", key);
    let fs = ctx.feature_state;

    let sorting_selector = combo_box(
        &fs.sorting_state,
        local("sorting_placeholder"),
        Some(&fs.active_criterion),
        |s| Message::SetCriterion(s).into(),
    );

    let input_section: Element<'a, GlobalMessage> = match &fs.active_criterion {
        Criterion::ByExtension => {
            text_input(local("extension"), &fs.extension_input.clone().unwrap_or_default())
                .on_input(|i| Message::ExtensionInput(i).into())
                .into()
        }
        Criterion::ByName => column![
            text_input(local("starts_with"), &fs.starts_with_input.clone().unwrap_or_default())
                .on_input(|i| Message::StartsWithInput(i).into()),
            text_input(local("ends_with"), &fs.ends_with_input.clone().unwrap_or_default())
                .on_input(|i| Message::EndsWithInput(i).into()),
            text_input(local("contains"), &fs.contains_input.clone().unwrap_or_default())
                .on_input(|i| Message::ContainsInput(i).into()),
        ]
        .padding(COL_PADDING)
        .spacing(COL_SPACING)
        .into(),
    };

    let destination_input =
        text_input(local("destination"), &fs.destination_input.clone().unwrap_or_default())
            .on_input(|i| Message::DestinationInput(i).into());

    let controls = row![
        button(text(local("apply_btn")))
            .width(Length::Fill)
            .style(button::success)
            .on_press(Message::ApplyRuleEdit(fs.focused_rule).into()),
        button(text(local("cancel_btn")))
            .width(Length::Fill)
            .style(button::danger)
            .on_press(Message::CancelEdit.into()),
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
    .style(container::bordered_box)
    .padding(CONTAINER_PADDING)
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
            let bg = focused.then_some(palette.primary.base.color).unwrap_or(Color::TRANSPARENT);
            let interaction = ctx
                .feature_state
                .rule_mode
                .then_some(mouse::Interaction::NotAllowed)
                .unwrap_or(mouse::Interaction::Pointer);
            let rule_idx = (!ctx.feature_state.rule_mode).then_some(rule_idx);

            mouse_area(
                container(rule_text(rule, focused, locale, palette))
                    .padding(CONTAINER_PADDING)
                    .style(move |theme| container::bordered_box(theme).background(bg)),
            )
            .interaction(interaction)
            .on_press(Message::FocusRule(rule_idx).into())
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
        focused.then_some(palette.primary.base.text).unwrap_or(palette.primary.base.color);
    let secondary_color =
        focused.then_some(palette.success.base.text).unwrap_or(palette.success.base.color);
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
