//! Deserializer for urlencoded form data.
//!
//! The functions [`from_bytes`] and [`from_str`] are the main entry points used
//! for using the deserializer.

use std::borrow::Cow;

use serde::{de, forward_to_deserialize_any};
use de::IntoDeserializer;
use de::value::MapDeserializer;

mod encoding;
use encoding::{Part, PairIter};

mod error;
pub use error::Error;

type Result<T> = std::result::Result<T, Error>;

/// Deserializer for form urlencoded data.
///
/// It is preferrable to use the helper methods [`from_bytes`] and [`from_str`]
/// instead of using this struct directly
pub struct Deserializer<'de>(MapDeserializer<'de, PairIter<'de>, Error>);

impl<'de> Deserializer<'de> {
    /// Construct an form urlencoded deserializer with the provided bytes
    #[must_use]
    pub fn new(bytes: &'de [u8]) -> Self {
        Self(MapDeserializer::new(PairIter(bytes)))
    }
}

/// Deserialize an instance of type `T` from bytes of form urlencoded data
pub fn from_bytes<'de, T: de::Deserialize<'de>>(bytes: &'de [u8]) -> Result<T> {
    T::deserialize(Deserializer::new(bytes))
}

/// Deserialize an instance of type `T` from a string of form urlencoded data
pub fn from_str<'de, T: de::Deserialize<'de>>(str: &'de str) -> Result<T> {
    T::deserialize(Deserializer::new(str.as_bytes()))
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_map(self.0)
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(self.0)
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(self.0)
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.0.end()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_option<V: de::Visitor<'de>>(
        self,
        visitor: V
    ) -> Result<V::Value> {
        match self.0.clone().end() {
            Ok(()) => visitor.visit_none(),
            Err(_) => visitor.visit_some(self),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 char str string
        bytes byte_buf tuple_struct struct identifier enum ignored_any
    }
}

impl<'de> IntoDeserializer<'de, Error> for Part<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self { self }
}

macro_rules! forward_parsed_value {
    ($($method:ident($t:ty) -> $var:ident;)*) => {
        $(
            fn $method<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
                let str = str::from_utf8(self.value()).map_err(Error::Utf8)?;
                match str.parse::<$t>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(Error::$var(e))
                }
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for Part<'de> {
    type Error = Error;
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.inner() {
            Cow::Borrowed(value) => visitor.visit_borrowed_bytes(value),
            Cow::Owned(value) => visitor.visit_byte_buf(value),
        }
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let str = str::from_utf8(self.value()).map_err(Error::Utf8)?;
        let mut chars = str.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => visitor.visit_char(ch),
            _ => visitor.visit_str(str),
        }
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(self)
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    forward_parsed_value! {
        deserialize_bool(bool) -> ParseBool;
        deserialize_u8(u8) -> ParseInt;
        deserialize_u16(u16) -> ParseInt;
        deserialize_u32(u32) -> ParseInt;
        deserialize_u64(u64) -> ParseInt;
        deserialize_u128(u128) -> ParseInt;
        deserialize_i8(i8) -> ParseInt;
        deserialize_i16(i16) -> ParseInt;
        deserialize_i32(i32) -> ParseInt;
        deserialize_i64(i64) -> ParseInt;
        deserialize_i128(i128) -> ParseInt;
        deserialize_f32(f32) -> ParseFloat;
        deserialize_f64(f64) -> ParseFloat;
    }

    forward_to_deserialize_any! {
        str string unit bytes byte_buf unit_struct tuple_struct struct
        identifier tuple ignored_any seq map
    }
}

impl<'de> de::EnumAccess<'de> for Part<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
        where V: de::DeserializeSeed<'de>
    {
        let variant = seed.deserialize(self.value().into_deserializer())?;
        Ok((variant, self))
    }
}

impl<'de> de::VariantAccess<'de> for Part<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
        where T: de::DeserializeSeed<'de>
    {
        Err(Error::UnitVariant)
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        Err(Error::UnitVariant)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        Err(Error::UnitVariant)
    }
}
