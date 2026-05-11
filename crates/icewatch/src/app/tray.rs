use iced::futures;
use iced::futures::SinkExt;
use iced::stream;
use tray_icon::MouseButton;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;

use crate::app::message::AppMessage;
use crate::app::message::Message;
use crate::app::message::SystemMessage;
use crate::app::state::Window;

pub(crate) fn tray_stream() -> impl futures::Stream<Item = Message> {
    stream::channel(100, |mut tx: futures::channel::mpsc::Sender<Message>| async move {
        let rx = TrayIconEvent::receiver();
        loop {
            // Block a thread-pool thread until an event arrives — no polling delay
            let event = smol::unblock(|| rx.recv()).await;
            match event {
                Ok(TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }) => {
                    let _ = tx.send(Message::App(AppMessage::View(Window::Main))).await;
                }
                Ok(_) => {}
                Err(_) => break, // sender dropped, tray gone
            }
        }
    })
}

pub(crate) fn tray_menu_stream(ids: Vec<String>) -> impl futures::Stream<Item = Message> {
    stream::channel(100, |mut tx: futures::channel::mpsc::Sender<Message>| async move {
        let rx = MenuEvent::receiver();
        loop {
            // Block a thread-pool thread until an event arrives — no polling delay
            let event = smol::unblock(|| rx.recv()).await;
            match event {
                Ok(MenuEvent { id }) => {
                    if ids.iter().any(|stored| *stored == id.0) {
                        let _ = tx.send(Message::System(SystemMessage::Exit)).await;
                    }
                }
                Err(_) => break, // sender dropped, tray gone
            }
        }
    })
}
