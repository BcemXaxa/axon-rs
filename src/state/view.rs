use crate::icons::Icon;

use super::update::*;
use super::*;

use iced::alignment::Vertical;
use iced::widget::container::bordered_box;
use iced::widget::text::Wrapping;
use iced::widget::{column, row, *};
use iced::*;

impl State {
    pub fn view(&self) -> Element<Message> {
        let height = row![
            container(self.tab_bar()).padding(Padding::ZERO.top(10)),
            container(self.working_file()).padding(Padding::ZERO.top(7.5)),
            container(self.actions()).padding(Padding::ZERO.top(10))
        ]
        .padding([0.0, 10.0])
        .height(Shrink);
        let top_bar = height;
        let main = container(self.main()).height(FillPortion(2));
        let warnings = self.errors();
        column![top_bar, column![main, warnings].spacing(10)]
            .padding(Padding::new(10.0).top(0))
            .into()
    }
    fn tab_bar(&self) -> Element<Message> {
        let tab_editing = button(row![Icon::Edit.as_text(), text("Editor")].spacing(10))
            .on_press_maybe({
                let if_active = true;
                if_active.then_some(Message::OpenTab(TabBar::Editor))
            })
            .style(|theme, status| tab_style(theme, status, self.tab == TabBar::Editor));

        let tab_plotting = button(row![Icon::Chart.as_text(), text("Plotter")].spacing(10))
            .on_press_maybe({
                let if_active = true;
                if_active.then_some(Message::OpenTab(TabBar::Plotter))
            })
            .style(|theme, status| tab_style(theme, status, self.tab == TabBar::Plotter));
        let tab_converting = button(row![Icon::Forward.as_text(), text("Converter")].spacing(10))
            .on_press_maybe({
                let if_active = true;
                if_active.then_some(Message::OpenTab(TabBar::Converter))
            })
            .style(|theme, status| tab_style(theme, status, self.tab == TabBar::Converter));

        row![tab_editing, tab_plotting, tab_converting]
            .spacing(5)
            .width(Fill)
            .into()
    }
    fn main(&self) -> Element<Message> {
        let file_select = self.file.is_none().then(|| self.file_select());
        let main = file_select.unwrap_or_else(|| match self.tab {
            TabBar::Editor => self.editor(),
            TabBar::Plotter => self.plotter(),
            TabBar::Converter => self.converter(),
        });
        if let Some(overlay) = self.file_hovered_overlay() {
            widget::stack!(main, overlay).into()
        } else {
            main
        }
    }
    fn file_hovered_overlay(&self) -> Option<Element<Message>> {
        self.file_hovered.as_ref().map(|path| {
            let file_is_valid = path.is_file() && path.extension().is_some_and(|ext| ext == "amcx");
            let description: Element<_> = if file_is_valid {
                column![
                    text("Open file"),
                    text(path.file_name().unwrap().to_str().unwrap())
                ]
                .align_x(Center)
                .into()
            } else {
                column![
                    Icon::Error.as_text(),
                    text("Invalid file!"),
                    text("Nothing will be opened")
                ]
                .align_x(Center)
                .into()
            };
            let wrapped = container(description).style(bordered_box).padding(20);
            let dimmed = container(wrapped)
                .center(Fill)
                .style(|theme| {
                    let color = theme.extended_palette().background.weak.color;
                    container::Style {
                        background: Some(color.scale_alpha(0.5).into()),
                        ..bordered_box(theme)
                    }
                })
                .into();
            dimmed
        })
    }
    fn actions(&self) -> Element<Message> {
        let actions = match self.tab {
            TabBar::Editor => self.editor_actions(),
            TabBar::Plotter => self.plot_actions(),
            TabBar::Converter => self.converting_actions(),
        };
        container(actions).align_right(Fill).into()
    }
    fn working_file(&self) -> Element<Message> {
        let title = match &self.file {
            Some(file) => {
                let mut title = file.path.file_name().unwrap().to_str().unwrap().to_owned();
                file.modified.then(|| title.push('*'));
                text(title)
            }
            None => text("No file selected"),
        };
        button(title)
            .on_press(FileMessage::Dialog(Action::Open).into())
            .style(|theme, status| {
                let background = button::secondary(theme, status).background;
                let border = container::bordered_box(theme).border.rounded(5);
                button::Style {
                    background,
                    border,
                    text_color: Color::WHITE,
                    ..Default::default()
                }
            })
            .padding([2.5, 20.0])
            .into()
    }
    fn editor_actions(&self) -> Element<Message> {
        let open = button(row![Icon::Open.as_text(), "Open"].spacing(10)).on_press_maybe({
            let if_active = !self.dialog;
            if_active.then_some(FileMessage::Dialog(Action::Open).into())
        });

        let mut path = None;
        let mut modified = false;
        if let Some(file) = &self.file {
            path = Some(file.path.to_owned());
            modified = file.modified;
        }

        let save = button(row![Icon::Save.as_text(), "Save"].spacing(10)).on_press_maybe({
            let msg = path.map(|path| FileMessage::Save(path).into());
            let if_active = !self.dialog && modified;
            if_active.then_some(msg).flatten()
        });

        let save_as = button(text("Save as..")).on_press_maybe({
            let if_active = !self.dialog && modified;
            if_active.then_some(FileMessage::Dialog(Action::Save).into())
        });

        row![open, save, save_as].spacing(5).into()
    }
    fn plot_actions(&self) -> Element<Message> {
        button(row![Icon::Menu.as_text(), "Select"].spacing(10))
            .on_press_maybe({
                let if_active = self.charts.is_some();
                if_active.then_some(PlottingMessage::SwitchSelector.into())
            })
            .style(|theme, status| {
                let activated = self.selector_visible;
                let mut style = button::primary(theme, status);
                use button::Status as S;
                match status {
                    S::Active if activated => {
                        let color = mix(theme.palette().primary, Color::WHITE, 0.3);
                        style.background = Some(color.into());
                    }
                    S::Hovered | S::Pressed if activated => {
                        let color = mix(theme.palette().primary, Color::WHITE, 0.1);
                        style.background = Some(color.into());
                    }
                    _ => (),
                }
                style
            })
            .into()
    }
    fn converting_actions(&self) -> Element<Message> {
        Space::new(Fill, Fill).into()
    }
    fn editor(&self) -> Element<Message> {
        let File { path, content, .. } = self.file.as_ref().unwrap();

        let mut editor = text_editor(content)
            .height(Fill)
            .font(Font::MONOSPACE)
            .wrapping(Wrapping::None);
        if !self.dialog {
            editor = editor.on_action(|action| FileMessage::Edit(action).into())
        }

        let coords = content.cursor_position();
        let status_bar = row![
            text(path.to_str().unwrap_or("")).width(Length::Fill),
            text(format!("{} : {}", coords.0 + 1, coords.1 + 1))
        ]
        .padding([0.0, 5.0])
        .height(Length::Fixed(30.0))
        .align_y(Vertical::Center);

        container(column![status_bar, editor])
            .padding(5)
            .style(bordered_box)
            .into()
    }

    fn plotter(&self) -> Element<Message> {
        let plot = if let Some(Charts {
            sensors,
            selected: Some(selected),
            charts,
        }) = &self.charts
        {
            let selected_id = selected.id;
            let (acc_chart, gyr_chart) = charts.get(selected_id).unwrap();
            let charts = column![acc_chart.view(), gyr_chart.view()]
                .spacing(10)
                .width(FillPortion(3));

            if self.selector_visible {
                let selector = pick_list(sensors.clone(), Some(selected), |item| {
                    PlottingMessage::Sensor(item).into()
                })
                .width(FillPortion(1));
                row![charts, selector].spacing(10).into()
            } else {
                charts.into()
            }
        } else {
            self.nothing_to_display()
        };
        container(plot).padding(10).style(bordered_box).into()
    }

    fn converter(&self) -> Element<Message> {
        container(self.nothing_to_display())
            .style(bordered_box)
            .into()
    }

    fn file_select(&self) -> Element<Message> {
        let explore = button(text("Explore")).on_press_maybe({
            let if_active = !self.dialog;
            if_active.then_some(FileMessage::Dialog(Action::Open).into())
        });

        let file_select = column![
            text("No file selected!"),
            text("Drag and drop"),
            Icon::Drop.as_text(),
            text("or"),
            explore,
        ]
        .align_x(Center)
        .spacing(10);

        container(file_select)
            .center(Fill)
            .style(bordered_box)
            .into()
    }

    fn errors(&self) -> Element<Message> {
        let top = row![
            button(Icon::Error.as_text())
                .on_press(ErrorMessage::Expand.into())
                .style(|theme, status| {
                    match self.errors.unread {
                        true => button::danger(theme, status),
                        false => button::primary(theme, status),
                    }
                }),
            container(
                text(format!("Errors: {}", self.errors.list.len()))
                    .height(Fill)
                    .center()
            )
            .padding(Padding::ZERO.right(5).left(10))
        ]
        .height(Shrink);
        let top = top.push_maybe(
            self.errors.expanded.then_some(
                container(
                    button(row![Icon::Trash.as_text(), text("Clear")].spacing(10))
                        .on_press(ErrorMessage::Clear.into()),
                )
                .align_right(Fill),
            ),
        );

        if self.errors.expanded {
            let errors = self.errors.list.iter().map(|err| {
                container(text(err))
                    .width(Fill)
                    .padding(10)
                    .style(bordered_box)
                    .into()
            });
            let errors = column(errors).spacing(5).padding([5, 0]);
            let errors = scrollable(errors).width(Fill).height(Fill).spacing(5);

            container(column![top, errors].spacing(5))
                .width(Fill)
                .height(Fill)
                .padding(5)
                .style(bordered_box)
                .into()
        } else {
            let left_menu = container(top)
                .width(Shrink)
                .height(Shrink)
                .padding(5)
                .style(bordered_box);

            let right_menu = container(
                button(row![Icon::GitHub.as_text(), "GitHub"].spacing(10)).on_press(
                    Message::OpenWeb("https://github.com/BcemXaxa/axon-rs".into()),
                ),
            )
            .padding(5)
            .style(bordered_box);

            row![left_menu, Space::with_width(Fill), right_menu].into()
        }
    }
    fn nothing_to_display(&self) -> Element<Message> {
        container("Nothing to display").center(Fill).into()
    }
}

fn mix(c1: Color, c2: Color, k: f32) -> Color {
    let f = 1.0 - k;
    Color {
        r: c1.r * f + c2.r * k,
        g: c1.g * f + c2.g * k,
        b: c1.b * f + c2.b * k,
        a: c1.a * f + c2.a * k,
    }
}

fn tab_style(theme: &Theme, status: button::Status, activated: bool) -> button::Style {
    let mut style = button::primary(theme, status);
    if activated {
        let color = mix(theme.palette().primary, Color::WHITE, 0.3);
        style.background = Some(color.into());
    }
    style
}
