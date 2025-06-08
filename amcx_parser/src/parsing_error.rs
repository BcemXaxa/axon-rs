use std::{fmt::Display, num::ParseIntError};

use amcx_core::raw::ConfigKey;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct ParsingError {
    line: Option<usize>,
    inner: InnerParsingError,
}
impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = &self.line {
            write!(f, "Line {line}: ")?
        }
        self.inner.fmt(f)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum InnerParsingError {
    #[error("expected {0}, but found nothing")]
    TokenExpected(String),
    #[error("expected {expected}, but found {found}")]
    TokenUnexpected { expected: String, found: String },
    #[error("duplicate references are not allowed: {0}")]
    SensorNameDuplicate(String),
    #[error("duplicate configs are not allowed: {0}")]
    ConfigDuplicate(&'static str),
    #[error("missing required config: {0}")]
    ConfigMissing(&'static str),
    #[error("unknown config key: {0}, valid keys are: {keys:?}", keys = ConfigKey::ALL_KEYS)]
    ConfigUnknownKey(String),
    #[error("unsupported value {value} for key {key}, valid values are: {valid_values:?}")]
    ConfigUnsupportedValue {
        value: String,
        key: String,
        valid_values: Vec<&'static str>,
    },
    #[error(transparent)]
    NumberParsing(#[from] ParseIntError),
}
impl InnerParsingError {
    pub fn at(self, line: usize) -> ParsingError {
        ParsingError {
            line: Some(line),
            inner: self,
        }
    }
}
impl Into<ParsingError> for InnerParsingError {
    fn into(self) -> ParsingError {
        ParsingError {
            line: None,
            inner: self,
        }
    }
}
