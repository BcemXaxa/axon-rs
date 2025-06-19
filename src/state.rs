use std::{
    cell::LazyCell,
    io::Write,
    path::{Path, PathBuf},
    ptr::write_bytes,
    sync::LazyLock,
};

use amcx_core::Model;
use charts::{ChartSensor, SensorID};
use iced::widget::text_editor;

mod charts;
mod update;
mod view;

pub use update::*;

use crate::default_models::{self, DefaultModels};

pub struct State {
    tab: TabBar,
    dialog: bool,
    file_hovered: Option<PathBuf>,
    file: Option<File>,
    model: Option<Model>,
    charts: Option<Charts>,
    anim_model: AnimModel,
    chosen_model: DefaultModels,
    converted: Option<Converted>,
    selector_visible: bool,
    errors: Errors,
}
impl Default for State {
    fn default() -> Self {
        State {
            anim_model: default_models::HUMAN.clone(),
            chosen_model: DefaultModels::Human,
            tab: TabBar::Editor,
            dialog: false,
            file_hovered: None,
            file: None,
            model: None,
            charts: None,
            converted: None,
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

pub struct Converted {
    modified: AnimModel,
}

#[derive(Clone)]
pub struct AnimModel {
    pub gltf: gltf::json::Root,
    pub bins: Vec<(String, Vec<u8>)>,
}

impl AnimModel {
    fn write_to(&self, path: &Path) -> Result<(), std::io::Error> {
        let gltf_file = std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        self.gltf.to_writer_pretty(gltf_file)?;

        let path = path.parent().unwrap();
        for (name, bin) in &self.bins {
            let mut bin_file = std::fs::File::options()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path.join(name))?;
            bin_file.write_all(bin)?
        }
        Ok(())
    }
}
