use iced::Task;

use super::super::ContextMut;
use crate::app::message::Message as GlobalMessage;

/// Represents a message from the journal view.
#[derive(Debug, Clone)]
pub(crate) enum JournalMessage {}

impl JournalMessage {
    pub(crate) fn update<'a>(self, _ctx: ContextMut<'a>) -> Task<GlobalMessage> {
        Task::none()
    }
}
