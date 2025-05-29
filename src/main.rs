// #![cfg_attr(all(target_os = "windows", not(test)), windows_subsystem = "windows")]
// #![allow(unused)]

use std::future::Future;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::futures::FutureExt;
use iced::widget::container::bordered_box;
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, row, text, text_editor, Space};
use iced::Alignment::Center;
use iced::Length::{self, Fill, Shrink};
use iced::{application, window, Element, Font, Subscription, Task, Theme};

fn main() -> iced::Result {
    application("Axon", State::update, State::view)
        .antialiasing(true)
        .window_size([1600.0, 900.0])
        .centered()
        .theme(State::theme)
        .subscription(State::subscription)
        .run_with(State::new)
}
#[derive(Debug, Clone)]
enum LeftTab {
    Text,
    Plot,
}
struct State {
    theme: Theme,
    left_tab: LeftTab,
    dialog: bool,
    content: Option<(text_editor::Content, PathBuf)>,
}
impl State {
    fn new() -> (State, Task<Message>) {
        let state = State {
            theme: Theme::Nord,
            left_tab: LeftTab::Text,
            content: None,
            dialog: false,
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
                //E::FileHovered(path_buf) => ,
                E::FileDropped(path) => Message::OpenFile(path),
                //E::FilesHoveredLeft => ,
                _ => Message::None,
            }
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
        let row = row![
            self.left().into(),
            column![
                self.gltf_preview().into(),
                container(Space::new(Shrink, Shrink))
                    .center(Fill)
                    .style(bordered_box)
            ]
        ]
        .spacing(10);
        let row2 = row![
            row![
                button("Open..").on_press(Message::DialogOpen),
                button("Save").on_press_maybe(
                    self.content
                        .as_ref()
                        .map(|(_, path)| Message::SaveFile(path.to_owned()))
                ),
                button("Save as..")
                    .on_press_maybe(self.content.is_some().then_some(Message::DialogSave))
            ]
            .spacing(10)
            .width(Fill),
            container(button("Convert"))
                .align_x(Horizontal::Center)
                .width(Fill),
            container(button("Save"))
                .align_x(Horizontal::Right)
                .width(Fill),
        ];
        column![row2, row,].spacing(10).padding(10)
    }
}

// Elements
impl State {
    fn left(&self) -> impl Into<Element<Message>> {
        let left_content = if let Some((content, path)) = &self.content {
            match self.left_tab {
                LeftTab::Text => {
                    let editor = text_editor(content)
                        .height(Fill)
                        .font(Font::MONOSPACE)
                        .wrapping(Wrapping::None)
                        .on_action(Message::Edit);
                    let coords = content.cursor_position();
                    let status_bar = row![
                        text(path.to_str().unwrap_or("")).width(Length::Fill),
                        text(format!("{} : {}", coords.0 + 1, coords.1 + 1))
                    ]
                    .height(Length::Fixed(30.0))
                    .align_y(Vertical::Center);
                    Element::from(column![editor, status_bar])
                }
                LeftTab::Plot => Space::new(Fill, Fill).into(),
            }
        } else {
            self.file_select().into()
        };

        column![self.left_tabs().into(), left_content]
    }

    fn left_tabs(&self) -> impl Into<Element<Message>> {
        row![
            button(text("Text").center())
                .on_press(Message::OpenTab(LeftTab::Text))
                .width(Fill),
            button(text("Plot").center())
                .on_press(Message::OpenTab(LeftTab::Plot))
                .width(Fill),
        ]
    }

    fn file_select(&self) -> impl Into<Element<Message>> {
        let file_select = column![
            text("No file selected!"),
            text("Drag and drop"),
            text("or"),
            button("Explore").on_press(Message::DialogOpen),
        ]
        .align_x(Center)
        .spacing(10);

        container(file_select).center(Fill).style(bordered_box)
    }

    fn gltf_preview(&self) -> impl Into<Element<Message>> {
        container(button(text("GLTF preview").width(Fill).center())).width(Fill)
    }

    // fn top_menu(&self) -> impl Into<Element<Message>> {
    //     todo!()
    // }
}
#[derive(Debug, Clone)]
enum Message {
    None,

    DialogOpen,
    DialogOpenClosed,
    DialogSave,
    DialogSaveClosed,

    OpenFile(PathBuf),
    FileOpened(Arc<String>, PathBuf),
    SaveFile(PathBuf),
    FileSaved,

    OpenTab(LeftTab),
    Edit(text_editor::Action),
}
trait Update {
    type Message;
    fn update(&mut self, msg: Self::Message) -> Task<Self::Message>;
}

impl Update for State {
    type Message = Message;

    fn update(&mut self, msg: Self::Message) -> Task<Self::Message> {
        use Message as M;
        match msg {
            M::DialogOpen if !self.dialog => {
                self.dialog = true;
                return Task::future(dialog_open()).then(|maybe_path| {
                    let maybe_open = match maybe_path {
                        Some(path) => Task::done(Message::OpenFile(path)),
                        None => Task::none(),
                    };
                    Task::done(Message::DialogOpenClosed).chain(maybe_open) // FIXME for some reason doesn't work
                });
            }
            M::DialogOpenClosed => {
                self.dialog = false;
            }
            M::DialogSave if dbg!(!self.dialog) => {
                self.dialog = true;
                return Task::future(dialog_save()).then(|maybe_path| {
                    dbg!(&maybe_path);
                    let maybe_save = match maybe_path {
                        Some(path) => Task::done(Message::SaveFile(path)),
                        None => Task::none(),
                    };
                    Task::done(Message::DialogSaveClosed).chain(maybe_save) // FIXME for some reason doesn't work
                });
            }
            M::DialogSaveClosed => {
                self.dialog = false;
            }
            M::OpenFile(path) => {
                return Task::future(async {
                    match read_file(&path).await.map(Arc::new) {
                        Ok(content) => Message::FileOpened(content, path),
                        Err(_) => Message::None, // FIXME error handling
                    }
                });
            }
            M::SaveFile(path) if self.content.is_some() => {
                let content = self
                    .content
                    .as_ref()
                    .map(|(content, _)| content.text())
                    .unwrap();
                return Task::future(async move {
                    match write_file(&path, &content).await {
                        Err(_) => Message::None, // FIXME error handling
                        Ok(_) => Message::FileSaved,
                    }
                });
            }
            M::FileOpened(content, path) => {
                self.content = Some((text_editor::Content::with_text(&content), path));
            }
            M::OpenTab(left_tab) => {
                self.left_tab = left_tab;
            }
            M::Edit(action) if self.content.is_some() => {
                self.content.as_mut().unwrap().0.perform(action);
            }
            _ => (),
        }
        Task::none()
    }
}

fn dialog_open() -> impl Future<Output = Option<PathBuf>> {
    rfd::AsyncFileDialog::new()
        .set_title("Select motion capture data...")
        .add_filter("", &["amcx"])
        .pick_file()
        .map(|opt| opt.map(|fh| fh.path().to_path_buf()))
}

fn read_file(path: &Path) -> impl Future<Output = Result<String, std::io::Error>> + use<'_> {
    tokio::fs::read_to_string(path)
}

fn dialog_save() -> impl Future<Output = Option<PathBuf>> {
    rfd::AsyncFileDialog::new()
        .set_title("Select file to save to...")
        .add_filter("", &["amcx"])
        .set_can_create_directories(true)
        .save_file()
        .map(|opt| opt.map(|fh| fh.path().to_path_buf()))
}

fn write_file<'a>(
    path: &'a Path,
    content: &'a str,
) -> impl Future<Output = std::io::Result<()>> + use<'a> {
    tokio::fs::write(path, content)
}
