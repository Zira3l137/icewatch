use iced::Task;

use super::super::ContextMut;
use crate::app::features::main::Message;
use crate::app::message::Message as GlobalMessage;

/// Represents a message from the journal view.
#[derive(Debug, Clone)]
pub(crate) enum JournalMessage {
    JournalFilterInput(String),
    JournalFilterClear,
}

impl From<JournalMessage> for GlobalMessage {
    fn from(msg: JournalMessage) -> Self {
        Message::Journal(msg).into()
    }
}
impl JournalMessage {
    pub(crate) fn update<'a>(self, ctx: ContextMut<'a>) -> Task<GlobalMessage> {
        match self {
            JournalMessage::JournalFilterInput(term) => {
                ctx.feature_state.journal_filter = term;
            }
            JournalMessage::JournalFilterClear => {
                ctx.feature_state.journal_filter.clear();
            }
        }
        Task::none()
    }
}
