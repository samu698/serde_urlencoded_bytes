use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};
use std::str::{ParseBoolError, Utf8Error};

use serde::de;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
/// Error generated during deserialization of form urlencoded data
pub enum Error {
    /// Expected a unit variant.
    ///
    /// Only unit enum variants can be deserialized.
    UnitVariant,
    /// Invalid UTF8 sequence
    ///
    /// This error is generated only when deserializing a string, byte buffers
    /// accept any sequence of bytes
    Utf8(Utf8Error),
    /// Failed to parse boolean value
    ///
    /// Valid boolean values are `true` or `false`.
    ParseBool(ParseBoolError),
    /// Failed to parse integer value
    ParseInt(ParseIntError),
    /// Failed to parse floating point value
    ParseFloat(ParseFloatError),
    /// Error gerated by serde
    Serde(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnitVariant => write!(f, "Expected unit variant"),
            Self::Utf8(err) => write!(f, "Invalid UTF-8: {err}"),
            Self::ParseBool(err) => write!(f, "Invalid Bool: {err}"),
            Self::ParseInt(err) => write!(f, "Invalid Integer: {err}"),
            Self::ParseFloat(err) => write!(f, "Invalid Float: {err}"),
            Self::Serde(err) => err.fmt(f),
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
            Self::Serde(_) => None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Serde(format!("{msg}"))
    }
}
