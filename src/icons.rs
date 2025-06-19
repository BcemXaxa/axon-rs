use iced::{
    Font, Task,
    font::Error,
    widget::{Text, text},
};

pub enum Icon {
    Error,
    Forward,
    Open,
    Save,
    Close,
    Chart,
    Drop,
    Trash,
    GitHub,
    Edit,
    Menu,
}

impl Icon {
    const FONT: Font = Font::with_name("icons");
    pub fn as_text<'a>(self) -> Text<'a> {
        let code = match self {
            Icon::Error => '\u{E800}',
            Icon::Forward => '\u{E801}',
            Icon::Open => '\u{E802}',
            Icon::Save => '\u{E803}',
            Icon::Close => '\u{E804}',
            Icon::Chart => '\u{E805}',
            Icon::Drop => '\u{E807}',
            Icon::Trash => '\u{F1F8}',
            Icon::GitHub => '\u{F09B}',
            Icon::Edit => '\u{E806}',
            Icon::Menu => '\u{E808}',
        };
        Icon::from_code(code)
    }
    fn from_code<'a>(code: char) -> Text<'a> {
        text(code).font(Icon::FONT)
    }
}

pub fn load() -> Task<Result<(), Error>> {
    iced::font::load(include_bytes!("../assets/text-icons.ttf"))
}
