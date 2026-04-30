use std::time::Instant;

use iced::keyboard::Event as KeyboardEvent;
use iced::mouse::Event as MouseEvent;
use iced::window::Id;
use icewatch_utils::command::Command;

use crate::app::state::FeatureMessage;
use crate::app::state::Window;

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
