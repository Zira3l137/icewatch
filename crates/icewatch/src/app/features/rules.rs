use super::{COL_PADDING, COL_SPACING, CONTAINER_PADDING, DEFAULT_THEME, ROW_PADDING, ROW_SPACING};
use std::collections::HashMap;

use crate::{
    app::{
        App,
        message::{InputEvent, Message as GlobalMessage},
        state::FeatureMessage,
    },
    rules::{ByExtension, ByName, CriterionKind, Rule},
};

use anyhow::Context as _;
use iced::{
    Color, Element, Length, Task, Theme, mouse,
    widget::{
        button, column, combo_box, container, mouse_area, rich_text, row, space, text, text::Span,
        text_input,
    },
    window::Id,
};
use icewatch_utils::locale::Locale;

#[derive(Debug, Clone, Default)]
pub(crate) enum Criterion {
    ByName,
    #[default]
    ByExtension,
}

impl std::fmt::Display for Criterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Criterion::ByName => write!(f, "By name"),
            Criterion::ByExtension => write!(f, "By extension"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    theme_state: combo_box::State<String>,
    locale_state: combo_box::State<String>,
    focused_rule: Option<usize>,
    sorting_state: combo_box::State<Criterion>,
    active_criterion: Criterion,
    extension_input: Option<String>,
    starts_with_input: Option<String>,
    ends_with_input: Option<String>,
    contains_input: Option<String>,
    destination_input: Option<String>,
    rule_mode: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Context<'a> {
    feature_state: &'a State,
    current_theme: &'a str,
    current_locale: &'a str,
    sorting_rules: &'a Vec<Rule>,
    themes: &'a HashMap<String, Theme>,
    locales: &'a HashMap<String, Locale>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(app: &'a App) -> Self {
        Self {
            feature_state: &app.features_state.rules,
            current_theme: &app.persistent_state.current_theme,
            current_locale: &app.persistent_state.current_locale,
            sorting_rules: &app.persistent_state.sorting_rules,
            themes: &app.app_state.themes,
            locales: &app.app_state.locales,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ContextMut<'a> {
    feature_state: &'a mut State,
    sorting_rules: &'a mut Vec<Rule>,
    themes: &'a mut HashMap<String, Theme>,
    locales: &'a mut HashMap<String, Locale>,
}

impl<'a> ContextMut<'a> {
    pub(crate) fn new(app: &'a mut App) -> Self {
        Self {
            feature_state: &mut app.features_state.rules,
            sorting_rules: &mut app.persistent_state.sorting_rules,
            themes: &mut app.app_state.themes,
            locales: &mut app.app_state.locales,
        }
    }
}

pub(crate) fn init(ctx: ContextMut<'_>) {
    ctx.feature_state.sorting_state =
        combo_box::State::new([Criterion::ByExtension, Criterion::ByName].into());
    ctx.feature_state.theme_state = combo_box::State::new(ctx.themes.keys().cloned().collect());
    ctx.feature_state.locale_state = combo_box::State::new(ctx.locales.keys().cloned().collect());
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    FocusRule(Option<usize>),
    RemoveRule(Option<usize>),
    EditRule(Option<usize>),
    SetCriterion(Criterion),
    ApplyRuleEdit(Option<usize>),
    ExtensionInput(String),
    StartsWithInput(String),
    EndsWithInput(String),
    ContainsInput(String),
    DestinationInput(String),
    CancelEdit,
    AddRule,
}

impl From<Message> for GlobalMessage {
    fn from(msg: Message) -> GlobalMessage {
        GlobalMessage::Feature(FeatureMessage::Rules(msg))
    }
}

pub(crate) fn update(msg: Message, ctx: ContextMut<'_>) -> Task<GlobalMessage> {
    match msg {
        Message::FocusRule(idx) => {
            ctx.feature_state.focused_rule = idx;
        }
        Message::RemoveRule(Some(idx)) => {
            ctx.sorting_rules.remove(idx);
        }
        Message::SetCriterion(c) => {
            ctx.feature_state.active_criterion = c;
        }
        Message::CancelEdit => {
            ctx.feature_state.rule_mode = false;
        }
        Message::EditRule(Some(idx)) => {
            if let Some(rule) = ctx.sorting_rules.get(idx) {
                match &rule.criterion {
                    CriterionKind::ByExtension(crit) => {
                        ctx.feature_state.extension_input = Some(crit.extensions.join(", "));
                        ctx.feature_state.active_criterion = Criterion::ByExtension;
                    }
                    CriterionKind::ByName(crit) => {
                        ctx.feature_state.starts_with_input = crit.starts_with.clone();
                        ctx.feature_state.ends_with_input = crit.ends_with.clone();
                        ctx.feature_state.contains_input = crit.contains.clone();
                        ctx.feature_state.active_criterion = Criterion::ByName;
                    }
                }
                ctx.feature_state.destination_input =
                    Some(rule.destination.to_string_lossy().into_owned());
                ctx.feature_state.rule_mode = true;
            }
        }
        Message::DestinationInput(dest) => ctx.feature_state.destination_input = Some(dest),
        Message::ExtensionInput(ext) => ctx.feature_state.extension_input = Some(ext),
        Message::StartsWithInput(s) => ctx.feature_state.starts_with_input = Some(s),
        Message::EndsWithInput(s) => ctx.feature_state.ends_with_input = Some(s),
        Message::ContainsInput(s) => ctx.feature_state.contains_input = Some(s),
        Message::ApplyRuleEdit(idx) => {
            let fs = ctx.feature_state;
            let rule = match &fs.active_criterion {
                Criterion::ByExtension => Rule::new(
                    ByExtension::new(fs.extension_input.clone().unwrap_or_default()),
                    &fs.destination_input.clone().unwrap_or_default(),
                ),
                Criterion::ByName => Rule::new(
                    ByName {
                        starts_with: fs.starts_with_input.clone(),
                        ends_with: fs.ends_with_input.clone(),
                        contains: fs.contains_input.clone(),
                    },
                    &fs.destination_input.clone().unwrap_or_default(),
                ),
            }
            .context("failed to create rule")
            .unwrap();
            match idx.and_then(|i| ctx.sorting_rules.get_mut(i)) {
                Some(existing) => *existing = rule,
                None => ctx.sorting_rules.push(rule),
            }
            fs.rule_mode = false;
        }
        Message::AddRule => {
            ctx.feature_state.extension_input = None;
            ctx.feature_state.starts_with_input = None;
            ctx.feature_state.ends_with_input = None;
            ctx.feature_state.contains_input = None;
            ctx.feature_state.destination_input = None;
            ctx.feature_state.focused_rule = None;
            ctx.feature_state.rule_mode = true;
        }
        // exhaustive: RemoveRule(None), EditRule(None) are no-ops
        _ => {}
    }
    Task::none()
}

pub(crate) fn view<'a>(ctx: Context<'a>, _window_id: Id) -> Element<'a, GlobalMessage> {
    let theme = ctx
        .themes
        .get(ctx.current_theme)
        .unwrap_or_else(|| ctx.themes.get(DEFAULT_THEME).unwrap_or(&iced::Theme::Dark));
    let palette = theme.extended_palette();
    let locale = ctx.locales.get(ctx.current_locale).expect("locale not found");
    let local = |key: &str| locale.get_string("rules", key);
    let no_rules = ctx.sorting_rules.is_empty();

    let control_panel: Element<'a, GlobalMessage> = container(
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
    .into();

    let rules_box: Element<'a, GlobalMessage> = if no_rules {
        space().into()
    } else {
        container(list_box(ctx.clone(), locale, palette))
            .style(container::bordered_box)
            .padding(CONTAINER_PADDING)
            .into()
    };

    container(
        column![control_panel, edit_panel(ctx.clone(), locale), rules_box,]
            .padding(ROW_PADDING)
            .spacing(ROW_SPACING),
    )
    .align_top(Length::Shrink)
    .padding(CONTAINER_PADDING)
    .style(container::bordered_box)
    .into()
}

pub(crate) fn input(input: &InputEvent) -> Task<GlobalMessage> {
    let _ = input;
    Task::none()
}

fn edit_panel<'a>(ctx: Context<'a>, locale: &'a Locale) -> Element<'a, GlobalMessage> {
    if !ctx.feature_state.rule_mode {
        return space().into();
    }

    let local = |key: &str| locale.get_string("rules", key);
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
    let local = |key: &str| locale.get_string("rules", key);
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
