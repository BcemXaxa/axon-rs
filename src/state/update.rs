use amcx_parser::{parse as amcx_parse, parsing_error::ParsingError};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use iced::{Task, widget::text_editor};

use crate::default_models::DefaultModels;

use super::*;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    DialogClosed,

    File(FileMessage),
    Plot(PlottingMessage),
    Converting(ConvertingMessage),
    OpenTab(TabBar),
    OpenWeb(String),

    Error(ErrorMessage),
}
impl Message {
    fn task(self) -> Task<Message> {
        Task::done(self)
    }
}

#[derive(Debug, Clone)]
pub enum FileMessage {
    Dialog(Action),

    Hovered(Option<PathBuf>),
    Dropped(PathBuf),

    Open(PathBuf),
    Save(PathBuf),

    Opened(Arc<String>, PathBuf),
    Saved(PathBuf),

    Edit(text_editor::Action),
}
impl Into<Message> for FileMessage {
    fn into(self) -> Message {
        Message::File(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Open,
    Save,
}

#[derive(Debug, Clone)]
pub enum PlottingMessage {
    Sensor(SensorID),
    SwitchSelector,
}
impl Into<Message> for PlottingMessage {
    fn into(self) -> Message {
        Message::Plot(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConvertingDialog {
    Save,
    Calibration,
}

#[derive(Debug, Clone)]
pub enum ConvertingMessage {
    Convert,
    Dialog(ConvertingDialog),
    Save(PathBuf),
    OpenCalibration(PathBuf),
    CalibrationOpened(Arc<String>, PathBuf),
    ModelSelected(DefaultModels),
}
impl Into<Message> for ConvertingMessage {
    fn into(self) -> Message {
        Message::Converting(self)
    }
}

#[derive(Debug, Clone)]
pub enum ErrorMessage {
    Expand,
    Clear,
    Occured(Arc<dyn std::error::Error + Send + Sync>),
}
impl Into<Message> for ErrorMessage {
    fn into(self) -> Message {
        Message::Error(self)
    }
}

trait ToTask {
    fn task(self) -> Task<Message>;
}
impl<M: Into<Message>> ToTask for M {
    fn task(self) -> Task<Message> {
        self.into().task()
    }
}

impl State {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DialogClosed => {
                self.dialog = false;
                Task::none()
            }
            Message::OpenTab(tab) => {
                self.tab = tab;
                match self.validate_tab_content() {
                    Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
                    Ok(_) => Task::none(),
                }
            }
            Message::OpenWeb(path) => match open::that(path) {
                Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
                Ok(_) => Task::none(),
            },
            Message::File(message) => self.file_message(message),
            Message::Plot(message) => self.plotting_message(message),
            Message::Converting(message) => self.converting_message(message),
            Message::Error(message) => self.error_message(message),
            _ => Task::none(),
        }
    }

    fn file_message(&mut self, file_message: FileMessage) -> Task<Message> {
        use ErrorMessage as Error;
        use FileMessage as File;
        match file_message {
            File::Dialog(action) if !self.dialog => {
                self.dialog = true;
                self.file_dialog(action).chain(Message::DialogClosed.task())
            }
            File::Hovered(maybe_file) => {
                self.file_hovered = maybe_file;
                Task::none()
            }
            File::Dropped(path) => {
                self.file_hovered = None;
                FileMessage::Open(path).task()
            }
            File::Open(path) => Task::future(async {
                match read_file(&path).await.map(Arc::new) {
                    Ok(content) => File::Opened(content, path).into(),
                    Err(err) => Error::Occured(Arc::new(err)).into(),
                }
            }),
            File::Save(path) => {
                let content = self.file.as_ref().unwrap().content.text();
                Task::future(async move {
                    match write_file(&path, &content).await {
                        Ok(_) => File::Saved(path).into(),
                        Err(err) => Error::Occured(Arc::new(err)).into(),
                    }
                })
            }
            File::Opened(content, path) => {
                let content = text_editor::Content::with_text(&content);
                self.file = Some(super::File {
                    path,
                    content,
                    modified: false,
                });
                match self.on_file_changed() {
                    Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
                    Ok(_) => Task::none(),
                }
            }
            File::Saved(new_path) => {
                let super::File { modified, path, .. } = self.file.as_mut().unwrap();
                *modified = false;
                *path = new_path;

                Task::none()
            }
            File::Edit(action) => {
                let content = &mut self.file.as_mut().unwrap().content;

                let is_edit = action.is_edit();
                content.perform(action);
                is_edit.then(|| self.file.as_mut().map(|f| f.modified = true));

                is_edit
                    .then(|| match self.on_file_changed() {
                        Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
                        Ok(_) => Task::none(),
                    })
                    .unwrap_or(Task::none())
            }
            _ => Task::none(),
        }
    }

    fn plotting_message(&mut self, message: PlottingMessage) -> Task<Message> {
        use PlottingMessage as Plot;
        match message {
            Plot::Sensor(sensor) => {
                let selected = &mut self.charts.as_mut().unwrap().selected;
                *selected = Some(sensor);
                Task::none()
            }
            Plot::SwitchSelector => {
                self.selector_visible = !self.selector_visible;
                Task::none()
            }
        }
    }

    fn converting_message(&mut self, message: ConvertingMessage) -> Task<Message> {
        use ConvertingMessage as Converting;
        match message {
            Converting::Convert => self.convert(),
            Converting::Dialog(action) => {
                self.convert_dialog(action)
                    .chain(Message::DialogClosed.task()) //.and_then(|path| Converting::Save(path).task())
            }
            Converting::OpenCalibration(path) => Task::future(async {
                match read_file(&path).await.map(Arc::new) {
                    Ok(content) => Converting::CalibrationOpened(content, path).into(),
                    Err(err) => ErrorMessage::Occured(Arc::new(err)).into(),
                }
            }),
            Converting::CalibrationOpened(content, path) => match amcx_parse(&content) {
                Ok(model) => {
                    self.calibration = Some((model, path));
                    Task::none()
                }
                Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
            },
            Converting::Save(path) if self.converted.is_some() => {
                match self.converted.as_ref().unwrap().modified.write_to(&path) {
                    Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
                    Ok(_) => Task::none(),
                }
            }
            Converting::ModelSelected(model) => {
                self.anim_model = model.get();
                self.chosen_model = model;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn error_message(&mut self, message: ErrorMessage) -> Task<Message> {
        use ErrorMessage as Error;
        match message {
            Error::Expand => {
                self.errors.expanded = !self.errors.expanded;
                if self.errors.expanded {
                    self.errors.unread = false;
                }
                Task::none()
            }
            Error::Occured(err) => {
                self.errors.list.push(err.to_string());
                self.errors.unread = !self.errors.expanded;
                Task::none()
            }
            Error::Clear => {
                self.errors.list.clear();
                Task::none()
            }
        }
    }

    fn file_dialog(&mut self, action: Action) -> Task<Message> {
        Task::future(dialog(action)).then(move |opt| match opt {
            None => Message::None.task(),
            Some(path) => match action {
                Action::Open => FileMessage::Open(path).task(),
                Action::Save => FileMessage::Save(path).task(),
            },
        })
    }
    fn convert_dialog(&mut self, action: ConvertingDialog) -> Task<Message> {
        Task::future(convert_dialog(action)).then(move |opt| match opt {
            None => Message::None.task(),
            Some(path) => match action {
                ConvertingDialog::Calibration => ConvertingMessage::OpenCalibration(path).task(),
                ConvertingDialog::Save => ConvertingMessage::Save(path).task(),
            },
        })
    }

    fn build_model(&mut self) -> Result<bool, ParsingError> {
        let maybe_source = self.file.as_ref().map(|f| f.content.text());
        if let Some(source) = maybe_source {
            self.model = Some(amcx_parse(&source)?);
            return Ok(true);
        }
        Ok(false)
    }
    fn build_charts(&mut self) -> Result<bool, ParsingError> {
        if self.build_model()? {
            let model = self.model.as_ref().unwrap();
            self.charts = Some(Charts::from_model(&model));
            return Ok(true);
        }
        Ok(false)
    }

    fn on_file_changed(&mut self) -> Result<(), ParsingError> {
        // invalidate previous state
        self.model = None;
        self.charts = None;
        self.converted = None;

        self.validate_tab_content()?;

        Ok(())
    }

    fn validate_tab_content(&mut self) -> Result<(), ParsingError> {
        match self.tab {
            TabBar::Editor => (),
            TabBar::Plotter => {
                self.build_charts()?;
            }
            TabBar::Converter => {
                self.build_model()?;
            }
        }
        Ok(())
    }

    fn convert(&mut self) -> Task<Message> {
        if self.converted.is_some() {
            return ConvertingMessage::Dialog(ConvertingDialog::Save).task();
        }
        let bin_name = "Animation.bin";
        match amcx_convert::to_gltf::convert(
            self.anim_model.gltf.clone(),
            bin_name,
            self.model.as_ref().unwrap(),
            self.calibration.as_ref().map(|c| &c.0),
        ) {
            Ok((new_gltf, bin)) => {
                let mut bins = self.anim_model.bins.clone();
                bins.push((bin_name.into(), bin));
                let converted = Converted {
                    modified: AnimModel {
                        gltf: new_gltf,
                        bins,
                    },
                };
                self.converted = Some(converted);
                ConvertingMessage::Dialog(ConvertingDialog::Save).task()
            }
            Err(err) => ErrorMessage::Occured(Arc::new(err)).task(),
        }
    }
}

fn dialog(action: Action) -> impl Future<Output = Option<PathBuf>> {
    let mut dialog = rfd::AsyncFileDialog::new();
    dialog = match action {
        Action::Open => dialog.set_title("Select motion capture data..."),
        Action::Save => dialog
            .set_title("Select file to save to...")
            .set_can_create_directories(true),
    };

    async move {
        match action {
            Action::Open => dialog.pick_file().await,
            Action::Save => dialog.save_file().await,
        }
        .map(|fh| fh.path().to_path_buf())
    }
}

fn read_file(path: &Path) -> impl Future<Output = Result<String, std::io::Error>> + use<'_> {
    tokio::fs::read_to_string(path)
}

fn write_file<'a>(
    path: &'a Path,
    content: &'a str,
) -> impl Future<Output = std::io::Result<()>> + use<'a> {
    tokio::fs::write(path, content)
}

fn convert_dialog(action: ConvertingDialog) -> impl Future<Output = Option<PathBuf>> {
    let mut dialog = rfd::AsyncFileDialog::new();
    dialog = match action {
        ConvertingDialog::Save => dialog
            .set_title("Select file to save to")
            .set_can_create_directories(true),
        ConvertingDialog::Calibration => dialog.set_title("Select calibration file"),
    };
    async move {
        match action {
            ConvertingDialog::Save => dialog.save_file().await,
            ConvertingDialog::Calibration => dialog.pick_file().await,
        }
        .map(|fh| fh.path().to_path_buf())
    }
}
