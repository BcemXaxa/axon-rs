// #![cfg_attr(all(target_os = "windows", not(test)), windows_subsystem = "windows")]
// #![allow(unused)]

use std::sync::Arc;

use iced::advanced::graphics::image::image_rs::ImageFormat;
use iced::{Subscription, Task, Theme, application, window};

use crate::state::*;

mod icons;
mod state;

fn main() -> iced::Result {
    application("Axon", State::update, State::view)
        .window(window::Settings {
            icon: Some(app_icon().unwrap()),
            min_size: Some([800.0, 600.0].into()),
            ..Default::default()
        })
        .antialiasing(true)
        .window_size([1600.0, 900.0])
        .centered()
        .theme(State::theme)
        .subscription(State::subscription)
        .run_with(State::new)
}
fn app_icon() -> Result<window::Icon, window::icon::Error> {
    iced::window::icon::from_file_data(include_bytes!("./assets/icon.png"), Some(ImageFormat::Png))
}

impl State {
    fn new() -> (State, Task<Message>) {
        let task = icons::load().map(|res| match res.map_err(|_| IconLoadError) {
            Ok(_) => Message::None,
            Err(err) => ErrorMessage::Occured(Arc::new(err)).into(),
        });
        let state = State::default();
        (state, task)
    }
    fn subscription(&self) -> Subscription<Message> {
        window::events().map(|(_id, event)| {
            use window::Event as E;
            match event {
                E::FileHovered(path) => FileMessage::Hovered(Some(path)).into(),
                E::FilesHoveredLeft => FileMessage::Hovered(None).into(),
                E::FileDropped(path) => FileMessage::Dropped(path).into(),
                _ => Message::None,
            }
        })
    }
    fn theme(_state: &State) -> Theme {
        Theme::Nord
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("Failed to load icons")]
struct IconLoadError;
