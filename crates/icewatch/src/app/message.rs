use std::time::Instant;

use crate::app::state::{FeatureMessage, Window};
use icewatch_utils::command::Command;

use iced::{keyboard::Event as KeyboardEvent, mouse::Event as MouseEvent, window::Id};

#[derive(Debug, Clone)]
pub enum Message {
    App(AppMessage),
    System(SystemMessage),
    Feature(FeatureMessage),
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    View(Window),
    Redraw(Instant),
    Hide(Id),
    Input(Id, InputEvent),
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
}

#[derive(Debug, Clone)]
pub enum SystemMessage {
    Execute(Command),
    Exit,
}
