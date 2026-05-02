use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};
use std::str::{ParseBoolError, Utf8Error};

use serde::de;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    UnitVariant,
    Utf8(Utf8Error),
    ParseBool(ParseBoolError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnitVariant => write!(f, "Expected unit variant"),
            Self::Utf8(err) => write!(f, "Invalid UTF-8: {err}"),
            Self::ParseBool(err) => write!(f, "Invalid Bool: {err}"),
            Self::ParseInt(err) => write!(f, "Invalid Integer: {err}"),
            Self::ParseFloat(err) => write!(f, "Invalid Float: {err}"),
            Self::Custom(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnitVariant => None,
            Self::Utf8(err) => Some(err),
            Self::ParseBool(err) => Some(err),
            Self::ParseInt(err) => Some(err),
            Self::ParseFloat(err) => Some(err),
            Self::Custom(_) => None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(format!("{msg}"))
    }
}
