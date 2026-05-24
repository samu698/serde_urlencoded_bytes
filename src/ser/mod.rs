//! Serializer for urlencoded form data.
//!
//! Values can be serialized into an owned result with [`to_string`] or 
//! [`to_vec`] and can be also appended to an existing value with 
//! [`append_string`] and [`append_vec`]

use serde::ser;

mod encoding;
use encoding::{Target, Encoder};

mod keyval;
use keyval::{KeySerializer, ValSerializer};

mod pair;
use pair::PairSerializer;

mod error;
pub use error::Error;

type Result<T> = std::result::Result<T, Error>;

type SerOk<T> = <T as ser::Serializer>::Ok;
type SerErr<T> = <T as ser::Serializer>::Error;

/// Serialize the provided value to a into a form urlencoded String
pub fn to_string<T: ?Sized + ser::Serialize>(value: &T) -> Result<String> {
    let encoder = Encoder::new(String::new());
    value.serialize(Serializer(encoder))
}

/// Append the result of serializing the value to the provided String, if the
/// serialization fails the string won't be modified
pub fn append_string<T: ?Sized + ser::Serialize>(
    string: &mut String,
    value: &T
) -> Result<()> {
    let encoder = Encoder::new(string);
    value.serialize(Serializer(encoder))?;
    Ok(())
}

/// Serialize the provided value to a into a form urlencoded `Vec<u8>`
pub fn to_vec<T: ?Sized + ser::Serialize>(value: &T) -> Result<Vec<u8>> {
    let encoder = Encoder::new(Vec::new());
    value.serialize(Serializer(encoder))
}

/// Append the result of serializing the value to the provided `Vec<u8>`, if the
/// serialization fails the string won't be modified
pub fn append_vec<T: ?Sized + ser::Serialize>(
    vec: &mut Vec<u8>,
    value: &T
) -> Result<()> {
    let encoder = Encoder::new(vec);
    value.serialize(Serializer(encoder))?;
    Ok(())
}

/// Serializer into form urlencoded data
pub struct Serializer<T: Target>(Encoder<T>);

macro_rules! invalid_toplevel {
    ($($name:ident$(<$T:ident>)?($($arg:ty),*) -> $ret:ident;)*) => {$(
        fn $name $(<$T: ?Sized + ser::Serialize>)? (
            self,
            $(_: $arg),*
        ) -> Result<Self::$ret> {
            Err(Error::TopLevel)
        }
    )*};
}

impl<T: Target> ser::Serializer for Serializer<T> {
    type Ok = T;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    invalid_toplevel! {
        serialize_bool(bool) -> Ok;
        serialize_i8(i8) -> Ok;
        serialize_i16(i16) -> Ok;
        serialize_i32(i32) -> Ok;
        serialize_i64(i64) -> Ok;
        serialize_i128(i128) -> Ok;
        serialize_u8(u8) -> Ok;
        serialize_u16(u16) -> Ok;
        serialize_u32(u32) -> Ok;
        serialize_u64(u64) -> Ok;
        serialize_u128(u128) -> Ok;
        serialize_f32(f32) -> Ok;
        serialize_f64(f64) -> Ok;
        serialize_char(char) -> Ok;
        serialize_str(&str) -> Ok;
        serialize_bytes(&[u8]) -> Ok;
        serialize_unit_variant(&'static str, u32, &'static str) -> Ok;
        serialize_newtype_variant<U>(&'static str, u32, &'static str, &U) -> Ok;
        serialize_tuple_struct(&'static str, usize) -> SerializeTupleStruct;
        serialize_tuple_variant(&'static str, u32, &'static str, usize) -> SerializeTupleVariant;
        serialize_struct_variant(&'static str, u32, &'static str, usize) -> SerializeStructVariant;
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.0.finish()
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> Result<Self::Ok> {
        self.0.finish()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.0.finish()
    }

    fn serialize_some<U: ?Sized + ser::Serialize>(
        self,
        value: &U,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_struct<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &U,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_seq(
        self,
        _len: Option<usize>
    ) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(
        self,
        _len: usize
    ) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeStruct> {
        Ok(self)
    }
}

impl<T: Target> ser::SerializeSeq for Serializer<T> {
    type Ok = SerOk<Self>;
    type Error = SerErr<Self>;

    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U
    ) -> Result<()> {
        value.serialize(PairSerializer::new(&mut self.0))
    }

    fn end(self) -> Result<Self::Ok> {
        self.0.finish()
    }
}

impl<T: Target> ser::SerializeTuple for Serializer<T> {
    type Ok = SerOk<Self>;
    type Error = SerErr<Self>;

    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U
    ) -> Result<()> {
        value.serialize(PairSerializer::new(&mut self.0))
    }

    fn end(self) -> Result<Self::Ok> {
        self.0.finish()
    }
}

impl<T: Target> ser::SerializeMap for Serializer<T> {
    type Ok = SerOk<Self>;
    type Error = SerErr<Self>;

    fn serialize_key<K: ?Sized + ser::Serialize>(
        &mut self,
        key: &K
    ) -> Result<()> {
        key.serialize(KeySerializer::new(&mut self.0))
    }

    fn serialize_value<V: ?Sized + ser::Serialize>(
        &mut self,
        value: &V
    ) -> Result<()> {
        value.serialize(ValSerializer::new(&mut self.0))
    }

    fn end(self) -> Result<Self::Ok> {
        self.0.finish()
    }
}

impl<T: Target> ser::SerializeStruct for Serializer<T> {
    type Ok = SerOk<Self>;
    type Error = SerErr<Self>;

    fn serialize_field<V: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &V
    ) -> Result<()> {
        self.0.push_key(key.as_bytes())?;
        value.serialize(ValSerializer::new(&mut self.0))
    }

    fn end(self) -> Result<Self::Ok> {
        self.0.finish()
    }
}
