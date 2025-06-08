use std::{path::PathBuf};


use amcx_core::Model;
use iced::widget::text_editor;
use charts::{SensorID, ChartSensor};

mod charts;
mod update;
mod view;

pub use update::*;

pub struct State {
    tab: TabBar,
    dialog: bool,
    file_hovered: Option<PathBuf>,
    file: Option<File>,
    model: Option<Model>,
    charts: Option<Charts>,
    selector_visible: bool,
    errors: Errors,
}
impl Default for State {
    fn default() -> Self {
        State {
            tab: TabBar::Editor,
            dialog: false,
            file_hovered: None,
            file: None,
            model: None,
            charts: None,
            selector_visible: true,
            errors: Errors {
                expanded: false,
                unread: false,
                list: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabBar {
    Editor,
    Plotter,
    Converter,
}

struct Errors {
    expanded: bool,
    unread: bool,
    list: Vec<String>,
}
struct File {
    path: PathBuf,
    modified: bool,
    content: text_editor::Content,
}

pub struct Charts {
    sensors: Vec<SensorID>,
    selected: Option<SensorID>,
    charts: Vec<(ChartSensor, ChartSensor)>,
}