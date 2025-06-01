// #![cfg_attr(all(target_os = "windows", not(test)), windows_subsystem = "windows")]
// #![allow(unused)]

use core::f64;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use amcx_parser::parsing_error::ParsingError;
use iced::Alignment::Center;
use iced::Length::{self, Fill, Shrink};
use iced::advanced::graphics::image::image_rs::ImageFormat;
use iced::alignment::{Horizontal, Vertical};
use iced::futures::FutureExt;
use iced::futures::channel::mpsc::{Sender, channel};
use iced::widget::container::bordered_box;
use iced::widget::shader::wgpu::naga::FastHashMap;
use iced::widget::text::Wrapping;
use iced::widget::{
    self, Space, button, column, combo_box, container, pick_list, row, text, text_editor,
};
use iced::window::Icon;
use iced::{Border, Element, Font, Shadow, Subscription, Task, Theme, application, color, window};

use amcx_core::*;
use amcx_parser::parse as amcx_parse;
use plotters::series::LineSeries;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

fn main() -> iced::Result {
    application("Axon", State::update, State::view)
        .antialiasing(true)
        .window(window::Settings {
            icon: Some(iced::window::icon::from_file_data(
                include_bytes!("./assets/icon.png"),
                Some(ImageFormat::Png),
            ).unwrap()),
            ..Default::default()
        })
        .window_size([1600.0, 900.0])
        .centered()
        .theme(State::theme)
        .subscription(State::subscription)
        .run_with(State::new)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftTab {
    Text,
    Plot,
}
struct State {
    theme: Theme,
    left_tab: LeftTab,
    dialog: bool,
    changed: bool,
    content: Option<(text_editor::Content, PathBuf)>,
    parsed: Option<Vec<Series>>,
    charts: Option<FastHashMap<String, (ChartSeries, ChartSeries)>>,
    selected_ref: Option<String>,
}
impl State {
    fn new() -> (State, Task<Message>) {
        let state = State {
            theme: Theme::Nord,
            left_tab: LeftTab::Text,
            content: None,
            dialog: false,
            changed: false,
            parsed: None,
            charts: None,
            selected_ref: None,
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
            self.left_open_save().into(),
            container(button("Convert"))
                .align_x(Horizontal::Center)
                .width(Fill),
            container(button("Save"))
                .align_x(Horizontal::Right)
                .width(Fill),
        ];
        column![row2, row].spacing(10).padding(10)
    }
}

// Elements
impl State {
    fn left(&self) -> impl Into<Element<Message>> {
        let left_content = if self.content.is_some() {
            match self.left_tab {
                LeftTab::Text => self.editor().into(),
                LeftTab::Plot => self.plot().into(),
            }
        } else {
            self.file_select().into()
        };

        column![self.left_tabs().into(), left_content]
    }

    fn left_open_save(&self) -> impl Into<Element<Message>> {
        let open = button("Open..").on_press_maybe({
            let if_active = !self.dialog;
            if_active.then_some(Message::DialogOpen)
        });

        let save = button("Save").on_press_maybe({
            let msg = self
                .content
                .as_ref()
                .map(|(_, path)| Message::SaveFile(path.to_owned()));

            let if_active = self.changed && !self.dialog;
            if_active.then_some(msg).flatten()
        });

        let save_as = button("Save as..").on_press_maybe({
            let if_active = self.changed && self.content.is_some() && !self.dialog;
            if_active.then_some(Message::DialogSave)
        });

        row![open, save, save_as].spacing(10).width(Fill)
    }

    fn editor(&self) -> impl Into<Element<Message>> {
        let (content, path) = self.content.as_ref().unwrap();

        let mut editor = text_editor(content)
            .height(Fill)
            .font(Font::MONOSPACE)
            .wrapping(Wrapping::None);
        if !self.dialog {
            editor = editor.on_action(Message::Edit)
        }

        let coords = content.cursor_position();
        let status_bar = row![
            text(path.to_str().unwrap_or("")).width(Length::Fill),
            text(format!("{} : {}", coords.0 + 1, coords.1 + 1))
        ]
        .height(Length::Fixed(30.0))
        .align_y(Vertical::Center);

        Element::from(column![editor, status_bar])
    }

    fn plot(&self) -> impl Into<Element<Message>> {
        if self.parsed.is_some() {
            let references: Vec<_> = self
                .parsed
                .as_ref()
                .unwrap()
                .iter()
                .map(|series| series.reference.clone())
                .collect();
            let pick_list =
                pick_list(references, self.selected_ref.as_ref(), Message::RefSelected).width(Fill);
            let charts = if let Some(selected) = &self.selected_ref {
                let (acc_chart, gyr_chart) = self.charts.as_ref().unwrap().get(selected).unwrap();
                let style = container::Style {
                    text_color: None,
                    background: Some(color!(40, 40, 50).into()),
                    border: Border::default(),
                    shadow: Shadow::default(),
                };
                let acc_chart = container(acc_chart.view()).style(move |_| style);
                let gyr_chart = container(gyr_chart.view()).style(move |_| style);
                column![acc_chart, gyr_chart].spacing(10).into()
            } else {
                Element::from(container(text("Select reference")).center(Fill))
            };
            column![pick_list, charts].spacing(10).into()
        } else {
            Element::from(Space::new(Fill, Fill))
        }
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
        let explore = button("Explore").on_press_maybe({
            let if_active = !self.dialog;
            if_active.then_some(Message::DialogOpen)
        });

        let file_select = column![
            text("No file selected!"),
            text("Drag and drop"),
            text("or"),
            explore,
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

    DialogOpen,
    DialogOpenClosed,
    DialogSave,
    DialogSaveClosed,

    OpenFile(PathBuf),
    FileOpened(Arc<String>, PathBuf),
    SaveFile(PathBuf),
    FileSaved(PathBuf),

    Parse,
    RefSelected(String),

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
                        Some(path) => Message::OpenFile(path),
                        None => Message::None,
                    };
                    Task::done(Message::DialogOpenClosed).chain(Task::done(maybe_open))
                });
            }
            M::DialogOpenClosed => {
                self.dialog = false;
            }
            M::DialogSave if !self.dialog => {
                self.dialog = true;
                return Task::future(dialog_save()).then(|maybe_path| {
                    let maybe_save = match maybe_path {
                        Some(path) => Message::SaveFile(path),
                        None => Message::None,
                    };
                    Task::done(Message::DialogSaveClosed).chain(Task::done(maybe_save))
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
                        Ok(_) => Message::FileSaved(path),
                    }
                });
            }
            M::FileOpened(content, path) => {
                self.content = Some((text_editor::Content::with_text(&content), path));
            }
            M::OpenTab(left_tab) => {
                self.left_tab = left_tab;
                if left_tab == LeftTab::Plot {
                    return Task::done(Message::Parse);
                }
            }
            M::Edit(action) if self.content.is_some() => {
                if action.is_edit() {
                    self.parsed = None;
                    self.charts = None;
                    self.selected_ref = None;
                    self.changed = true;
                }
                self.content.as_mut().unwrap().0.perform(action);
            }
            M::FileSaved(path) => {
                if let Some((_, local_path)) = self.content.as_mut() {
                    *local_path = path;
                }
                self.changed = false;
            }
            M::Parse if self.content.is_some() && self.parsed.is_none() => {
                let source = self.content.as_ref().unwrap().0.text();
                match amcx_parse(&source) {
                    Ok(parsed) => {
                        let charts = build_charts(&parsed);
                        self.charts = Some(charts);
                        self.parsed = Some(parsed);
                    }
                    Err(err) => (), // TODO: error handling
                }
            }
            M::RefSelected(reference) => self.selected_ref = Some(reference),
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

fn build_charts(parsed: &[Series]) -> FastHashMap<String, (ChartSeries, ChartSeries)> {
    let mut result = FastHashMap::default();

    for series in parsed {
        let samples = &series.samples;
        let (acc_samples, gyr_samples): (Vec<DataPoints>, Vec<DataPoints>) = samples
            .iter()
            .map(|sample| {
                let Sample {
                    dt,
                    acc_mps2,
                    gyr_dps,
                } = sample;
                ((dt, acc_mps2), (dt, gyr_dps))
            })
            .unzip();

        let acc_chart = ChartSeries::new(&acc_samples, CaptionType::Acc);
        let gyr_chart = ChartSeries::new(&gyr_samples, CaptionType::Gyr);

        result.insert(series.reference.clone(), (acc_chart, gyr_chart));
    }

    result
}

type DataPoints<'a> = (&'a Duration, &'a [f64; 3]);

enum CaptionType {
    Acc,
    Gyr,
}
impl CaptionType {
    fn str(&self) -> &'static str {
        match self {
            CaptionType::Acc => "Accelerometer",
            CaptionType::Gyr => "Gyroscope",
        }
    }
}

struct ChartSeries {
    caption: CaptionType,
    time_range: (f64, f64),
    axis_range: (f64, f64),
    line_x: Vec<(f64, f64)>,
    line_y: Vec<(f64, f64)>,
    line_z: Vec<(f64, f64)>,
}
impl ChartSeries {
    fn new(samples: &[DataPoints], caption: CaptionType) -> Self {
        let mut ts = Duration::ZERO;
        let mut line_x = Vec::with_capacity(samples.len());
        let mut line_y = Vec::with_capacity(samples.len());
        let mut line_z = Vec::with_capacity(samples.len());

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for (dt, [x, y, z]) in samples {
            ts += **dt;
            let ts = ts.as_secs_f64();

            line_x.push((ts, *x));
            line_y.push((ts, *y));
            line_z.push((ts, *z));

            min = min.min(*x).min(*y).min(*z);
            max = max.max(*x).max(*y).max(*z);
        }

        Self {
            caption,
            line_x,
            line_y,
            line_z,
            time_range: (0.0, ts.as_secs_f64()),
            axis_range: (min, max),
        }
    }
    fn view(&self) -> impl Into<Element<Message>> {
        ChartWidget::new(self).height(Fill).width(Fill)
    }
}
impl Chart<Message> for ChartSeries {
    type State = ();

    // TODO handle errors
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        use plotters::prelude::*;

        let x_spec = self.time_range.0..self.time_range.1;
        let y_spec = (self.axis_range.0 - 0.5)..(self.axis_range.1 + 0.5);
        let mut chart = builder
            .caption(
                self.caption.str(),
                ("sans-serif", 20).into_font().color(&WHITE),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(x_spec, y_spec)
            .expect("failed to build chart");
        chart
            .configure_mesh()
            .light_line_style(WHITE.mix(0.01))
            .bold_line_style(WHITE.mix(0.02))
            .axis_style(WHITE)
            .label_style(&WHITE)
            .draw()
            .expect("failed to draw mesh");

        let line_x = LineSeries::new(self.line_x.iter().cloned(), plotters::style::RED);
        let line_y = LineSeries::new(self.line_y.iter().cloned(), plotters::style::GREEN);
        let line_z = LineSeries::new(self.line_z.iter().cloned(), plotters::style::CYAN);

        chart.draw_series(line_x).expect("failed to draw series");
        chart.draw_series(line_y).expect("failed to draw series");
        chart.draw_series(line_z).expect("failed to draw series");
    }
}
