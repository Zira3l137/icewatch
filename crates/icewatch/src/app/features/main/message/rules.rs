use anyhow::Context as _;
use iced::Task;

use super::super::ContextMut;
use super::super::Message;
use super::super::data::Criterion;
use crate::app::message::Message as GlobalMessage;
use crate::rules::ByExtension;
use crate::rules::ByName;
use crate::rules::CriterionKind;
use crate::rules::Rule;

/// Represents a message from the rules view.
#[derive(Debug, Clone)]
pub(crate) enum RulesMessage {
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

impl From<RulesMessage> for GlobalMessage {
    fn from(msg: RulesMessage) -> Self {
        Message::Rules(msg).into()
    }
}

impl RulesMessage {
    pub(crate) fn update<'a>(self, ctx: ContextMut<'a>) -> Task<GlobalMessage> {
        match self {
            RulesMessage::FocusRule(idx) => {
                ctx.feature_state.focused_rule = idx;
            }
            RulesMessage::RemoveRule(Some(idx)) => {
                ctx.sorting_rules.remove(idx);
            }
            RulesMessage::SetCriterion(c) => {
                ctx.feature_state.active_criterion = c;
            }
            RulesMessage::CancelEdit => {
                ctx.feature_state.rule_mode = false;
            }
            RulesMessage::EditRule(Some(idx)) => {
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
            RulesMessage::DestinationInput(dest) => {
                ctx.feature_state.destination_input = Some(dest)
            }
            RulesMessage::ExtensionInput(ext) => ctx.feature_state.extension_input = Some(ext),
            RulesMessage::StartsWithInput(s) => ctx.feature_state.starts_with_input = Some(s),
            RulesMessage::EndsWithInput(s) => ctx.feature_state.ends_with_input = Some(s),
            RulesMessage::ContainsInput(s) => ctx.feature_state.contains_input = Some(s),
            RulesMessage::ApplyRuleEdit(idx) => {
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
            RulesMessage::AddRule => {
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
}
