#![doc = include_str!("../README.md")]

//..

pub mod de;
pub use de::{Deserializer, from_bytes, from_str};

pub mod ser;
pub use ser::{Serializer, append_string, append_vec, to_string, to_vec};
