use std::sync::Arc;

use iced::alignment::Vertical;
use iced::widget::{button, column, horizontal_space, row, text_editor::Content};
use iced::widget::{self, combo_box, container, text, text_editor, toggler};
use iced::{Element, Length, Task, Theme};

fn main() -> iced::Result {
    iced::application("There will be a cool program name", update, view)
        .theme(|state| state.theme.clone())
        .centered()
        .run_with(|| {
            let state = State {
                theme: Theme::Dark,

                communicator: Content::with_text(""),
            };
            (state, Task::none())
        })
}

fn theme_name(theme: &Theme) -> String {
    use Theme as T;
    match theme {
        T::Light => "Light",
        T::Dark => "Dark",
        T::Dracula => "Dracula",
        T::Nord => "Nord",
        T::SolarizedLight => "SolarizedLight",
        T::SolarizedDark => "SolarizedDark",
        T::GruvboxLight => "GruvboxLight",
        T::GruvboxDark => "GruvboxDark",
        T::CatppuccinLatte => "CatppuccinLatte",
        T::CatppuccinFrappe => "CatppuccinFrappe",
        T::CatppuccinMacchiato => "CatppuccinMacchiato",
        T::CatppuccinMocha => "CatppuccinMocha",
        T::TokyoNight => "TokyoNight",
        T::TokyoNightStorm => "TokyoNightStorm",
        T::TokyoNightLight => "TokyoNightLight",
        T::KanagawaWave => "KanagawaWave",
        T::KanagawaDragon => "KanagawaDragon",
        T::KanagawaLotus => "KanagawaLotus",
        T::Moonfly => "Moonfly",
        T::Nightfly => "Nightfly",
        T::Oxocarbon => "Oxocarbon",
        T::Ferra => "Ferra",
        T::Custom(_) => "Custom",
    }
    .into()
}

#[derive(Clone, Debug)]
enum Message {
    PreviousTheme,
    NextTheme,
    Clear,
    Move(text_editor::Action),
    Toggled(bool),
}

struct State {
    theme: Theme,

    communicator: Content,
}
impl State {
    fn log(&mut self, msg: impl Into<String>) {
        use text_editor::*;
        self.communicator.perform(Action::Move(Motion::DocumentEnd));
        if self.communicator.cursor_position().1 != 0 {
            self.communicator.perform(Action::Edit(Edit::Enter));
        }
        self.communicator
            .perform(Action::Edit(Edit::Paste(Arc::new(msg.into()))));
    }
}

fn update(state: &mut State, message: Message) -> impl Into<Task<Message>> {
    use Message as M;

    use text_editor::Action;
    use text_editor::Edit;
    match message {
        M::Clear => {
                        state.communicator.perform(Action::SelectAll);
                        state.communicator.perform(Action::Edit(Edit::Backspace));
            }
        M::Move(action) if !action.is_edit() => state.communicator.perform(action),
        M::Move(_) => state.log("You can't edit this"),
        M::PreviousTheme => {
                state.theme = Theme::ALL
                    .iter()
                    .cycle()
                    .skip_while(|theme| **theme != state.theme)
                    .skip(1)
                    .next()
                    .unwrap()
                    .clone();
            }
        M::NextTheme => {
                state.theme = Theme::ALL
                    .iter()
                    .rev()
                    .cycle()
                    .skip_while(|theme| **theme != state.theme)
                    .skip(1)
                    .next()
                    .unwrap()
                    .clone();
            }
    }
}

fn view(state: &State) -> Element<Message> {
    widget::
    let col = column![
        row![
            horizontal_space(),
            button("Clear log").on_press(Message::Clear)
        ]
        .spacing(10),
        text_editor(&state.communicator)
            .height(Length::Fill)
            .on_action(Message::Move),
        row![
            button("<").on_press(Message::PreviousTheme),
            button(">").on_press(Message::NextTheme),
            text(format!("Theme: {}", theme_name(&state.theme)))
                .align_y(Vertical::Center)
                .height(Length::Fill),
        ]
        .height(Length::Shrink)
        .spacing(10),
    ]
    .spacing(10);

    container(col).padding(10).into()
}
