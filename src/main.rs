#![windows_subsystem = "windows"]

use iced::alignment::Vertical;
use iced::overlay::menu::Menu;
use iced::widget::container::bordered_box;
use iced::widget::tooltip::Position;
use iced::widget::{
    self, button, column, combo_box, container, mouse_area, pick_list, rich_text, row, text, text_editor, tooltip, vertical_rule, Space
};
use iced::Alignment::{self, Center};
use iced::Length::{Fill, Shrink};
use iced::{application, border, window, Element, Subscription, Task, Theme};

fn main() -> iced::Result {
    application("Accelerometric modeling", Message::respond, State::view)
        .theme(State::theme)
        .subscription(State::subscription)
        .centered()
        .antialiasing(true)
        .run_with(State::new)
}
struct State {
    theme: Theme,
    is_changing: bool,
}
impl State {
    fn new() -> (State, Task<Message>) {
        let state = State {
            theme: Theme::Nord,
            is_changing: false,
        };
        (state, Task::none())
    }
    fn theme(state: &State) -> Theme {
        state.theme.clone()
    }
    fn subscription(&self) -> Subscription<Message> {
        window::events().map(|(_id, event)| {
            use window::Event as E;
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
impl State {
    fn view(&self) -> impl Into<Element<Message>> {
        let menu = Menu::new(state, options, hovered_option, on_selected, on_option_hovered, class);
        let top = container(row![].spacing(1).height(Shrink))
            .align_right(Fill)
            .style(bordered_box);
        column![
            top,
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
                    container(Space::new(Shrink, Shrink)).center(Fill).style(bordered_box)
                ]
            ]
            .spacing(10)
            .padding(10)
        ]
    }

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

#[derive(Debug, Clone)]
enum Message {
    None,
    ChangeTheme,
}
impl Message {
    fn respond(state: &mut State, msg: Self) -> Task<Message> {
        use Message as M;
        match msg {
            M::None => (),
            M::ChangeTheme => state.is_changing = true,
        }
        Task::none()
    }
}
