#![cfg_attr(
    all(target_os = "windows", not(test)),
    windows_subsystem = "windows"
)]

use iced::widget::container::bordered_box;
use iced::widget::{button, column, container, row, text, Space};
use iced::Alignment::Center;
use iced::Length::{Fill, Shrink};
use iced::{
    application, window, Element, Subscription, Task,
    Theme,
};

mod modules;

fn main() -> iced::Result {
    application("Accelerometric modeling", State::update, State::view)
        .theme(State::theme)
        .subscription(State::subscription)
        .centered()
        .antialiasing(true)
        .run_with(State::new)
}
struct State {
    theme: Theme,
}
impl State {
    fn new() -> (State, Task<Message>) {
        let state = State { theme: Theme::Nord };
        (state, Task::none())
    }
    fn theme(state: &State) -> Theme {
        state.theme.clone()
    }
    fn subscription(&self) -> Subscription<Message> {
        window::events().map(|(_id, event)| {
            //use window::Event as E;
            match event {
                // E::Opened { position, size } => todo!(),
                // E::Closed => todo!(),
                // E::Moved(point) => todo!(),
                // E::Resized(size) => todo!(),
                // E::RedrawRequested(instant) => todo!(),
                // E::CloseRequested => todo!(),
                // E::Focused => todo!(),
                // E::Unfocused => todo!(),
                // E::FileHovered(path_buf) => todo!(),
                // E::FileDropped(path_buf) => todo!(),
                // E::FilesHoveredLeft => todo!(),
                _ => (),
            }
            Message::None
        })
    }
}
trait View {
    type Message;
    fn view(&self) -> impl Into<Element<Self::Message>>;
}
impl View for State {
    type Message = Message;
    fn view(&self) -> impl Into<Element<Self::Message>> {
        column![
            row![
                column![
                    row![
                        button(text("Text").center()).width(Fill),
                        button(text("Plot").center()).width(Fill)
                    ],
                    self.file_select().into(),
                ]
                .height(Fill)
                .align_x(Center),
                column![
                    self.gltf_preview().into(),
                    container(Space::new(Shrink, Shrink))
                        .center(Fill)
                        .style(bordered_box)
                ]
            ]
            .spacing(10)
            .padding(10)
        ]
    }
}

// Elements
impl State {
    fn file_select(&self) -> impl Into<Element<Message>> {
        let file_select = column![
            text("No file selected!"),
            text("Drag and drop"),
            text("or"),
            button("Explore"),
        ]
        .align_x(Center)
        .spacing(10);

        container(file_select).center(Fill).style(bordered_box)
    }

    fn gltf_preview(&self) -> impl Into<Element<Message>> {
        container(button(text("GLTF preview").width(Fill).center())).width(Fill)
    }
}

trait Update {
    type Message;
    fn update(&mut self, msg: Self::Message) -> Task<Self::Message>;
}

#[derive(Debug, Clone)]
enum Message {
    None,
}
impl Update for State {
    type Message = Message;

    fn update(&mut self, msg: Self::Message) -> Task<Self::Message> {
        match msg {
            Message::None => Task::none(),
        }
    }
}
