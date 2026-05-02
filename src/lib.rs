//! Serde support for `x-www-form-urlencoded` format that allows fields with non
//! UTF-8 values

#![warn(clippy::todo)]

pub mod de;
pub use de::{Deserializer, from_bytes, from_str};

pub mod ser;
pub use ser::{Serializer, append_string, append_vec, to_string, to_vec};
