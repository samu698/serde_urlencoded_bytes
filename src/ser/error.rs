use std::fmt::Display;

use serde::ser;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    TopLevel,
    InvalidPair,
    InvalidPairLen,
    ExpectedKey,
    ExpectedValue,
    UnsupportedKeyValType,
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TopLevel => write!(f, "Only maps, structs and sequences of pairs are supported in the top-level"),
            Self::InvalidPair => write!(f, "A pair must be a tuple or a sequence"),
            Self::InvalidPairLen => write!(f, "Pair must have a length of two"),
            Self::ExpectedKey => write!(f, "Expected key"),
            Self::ExpectedValue => write!(f, "Expected value"),
            Self::UnsupportedKeyValType => write!(f, "Unspported key/value type"),
            Self::Custom(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(format!("{msg}"))
    }
}
