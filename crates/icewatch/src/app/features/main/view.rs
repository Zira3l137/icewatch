use iced::Element;
use iced::Length;
use iced::Theme;
use iced::alignment::Vertical;
use iced::widget::column;
use iced::widget::container;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::stack;
use icewatch_utils::locale::Locale;

use crate::app::features::COL_PADDING;
use crate::app::features::COL_SPACING;
use crate::app::features::CONTAINER_PADDING;
use crate::app::features::ROW_PADDING;
use crate::app::features::ROW_SPACING;
use crate::app::features::SCROLLBAR_SPACING;
use crate::app::features::main::Context;
use crate::app::features::main::data::JournalEntrySection;
use crate::app::features::main::elements::context_menu;
use crate::app::features::main::elements::dashboard;
use crate::app::features::main::elements::explorer;
use crate::app::features::main::elements::journal::filter_panel;
use crate::app::features::main::elements::toolbar;
use crate::app::main::elements::common::return_panel;
use crate::app::main::elements::journal::journal_entry_section;
use crate::app::main::elements::rules::control_panel;
use crate::app::main::elements::rules::edit_panel;
use crate::app::main::elements::rules::rules_panel;
use crate::app::message::Message as GlobalMessage;

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
                let toolbar = toolbar::toolbar(locale, ctx.clone());
                let dashboard = dashboard::dashboard(ctx.clone(), locale, theme);
                let explorer = explorer::explorer(ctx.clone(), locale, theme);
                let context_menu = if ctx.feature_state.context_menu_visible {
                    context_menu::context_menu(ctx.clone(), locale)
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
                .into()
            }
            MainView::Journal => {
                let journal = if ctx.feature_state.journal_filter.trim().is_empty() {
                    ctx.journal
                } else {
                    &ctx.journal.filtered(|path| {
                        path.components().any(|c| {
                            c.as_os_str()
                                .to_string_lossy()
                                .to_ascii_lowercase()
                                .contains(&ctx.feature_state.journal_filter)
                        })
                    })
                };

                let today_entries: Element<'a, GlobalMessage> =
                    journal_entry_section(JournalEntrySection::Today, journal, locale, palette);
                let yesterday_entries: Element<'a, GlobalMessage> =
                    journal_entry_section(JournalEntrySection::Yesterday, journal, locale, palette);
                let all_entries: Element<'a, GlobalMessage> =
                    journal_entry_section(JournalEntrySection::All, journal, locale, palette);

                let return_panel: Element<'a, GlobalMessage> = return_panel(locale);
                let filter_panel: Element<'a, GlobalMessage> = filter_panel(ctx.clone(), locale);
                let entries_panel: Element<'a, GlobalMessage> = container(
                    scrollable(column![today_entries, yesterday_entries, all_entries])
                        .spacing(SCROLLBAR_SPACING),
                )
                .height(Length::Shrink)
                .width(Length::Fill)
                .align_y(Vertical::Top)
                .padding(CONTAINER_PADDING)
                .style(container::bordered_box)
                .into();

                container(
                    column![return_panel, filter_panel, entries_panel]
                        .padding(ROW_PADDING)
                        .spacing(ROW_SPACING),
                )
                .align_top(Length::Shrink)
                .padding(CONTAINER_PADDING)
                .into()
            }
        }
    }
}
