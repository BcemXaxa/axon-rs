use std::{fmt::Display, num::ParseIntError};

use thiserror::Error;


#[derive(Error, Debug)]
pub struct ParsingError<'a> {
    line: Option<usize>,
    inner: InnerParsingError<'a>,
}
impl<'a> Display for ParsingError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = &self.line {
            write!(f, "Line {line}: ")?
        }
        self.inner.fmt(f)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum InnerParsingError<'a> {
    #[error("expected {0}, but found nothing")]
    TokenExpected(&'static str),
    #[error("expected {expected}, but found {found}")]
    TokenUnexpected {
        expected: &'static str,
        found: &'a str,
    },
    #[error("duplicate references are not allowed: {0}")]
    ReferenceDuplicate(String),
    #[error("duplicate configs are not allowed: {0}")]
    ConfigDuplicate(&'static str),
    #[error("missing required config: {0}")]
    ConfigMissing(&'static str),
    #[error("unknown config key: {0}")]
    ConfigUnknownKey(&'a str),
    #[error("unsupported value {value} for key {key}, valid values are: {valid_values}")]
    ConfigUnsupportedValue {
        value: &'a str,
        key: &'a str,
        valid_values: &'static str,
    },
    #[error(transparent)]
    NumberParsing(#[from] ParseIntError),
}
impl<'a> InnerParsingError<'a> {
    pub fn at(self, line: usize) -> ParsingError<'a> {
        ParsingError {
            line: Some(line),
            inner: self,
        }
    }
}
impl<'a> Into<ParsingError<'a>> for InnerParsingError<'a> {
    fn into(self) -> ParsingError<'a> {
        ParsingError {
            line: None,
            inner: self,
        }
    }
}