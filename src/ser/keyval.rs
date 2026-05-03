use std::marker::PhantomData;

use serde::ser;

use super::encoding::{Target, Encoder};
use super::{Error, Result};

pub type KeySerializer<'a, T> = KeyValSerializer<'a, T, Key>;
pub type ValSerializer<'a, T> = KeyValSerializer<'a, T, Val>;
pub struct KeyValSerializer<'a, T: Target, M: KeyValMode<T>>(
    &'a mut Encoder<T>,
    PhantomData<M>,
);

impl<'a, T: Target, M: KeyValMode<T>> KeyValSerializer<'a, T, M> {
    pub fn new(encoder: &'a mut Encoder<T>) -> Self {
        Self(encoder, PhantomData)
    }

    fn push(self, value: &[u8]) -> Result<()> {
        M::push(self.0, value)
    }

    fn push_str(self, value: &str) -> Result<()> {
        M::push(self.0, value.as_bytes())
    }

    fn push_int<I: itoa::Integer>(self, v: I) -> Result<()> {
        let mut buf = itoa::Buffer::new();
        let str = buf.format(v);
        self.push_str(str)
    }

    fn push_float<F: ryu::Float>(self, v: F) -> Result<()> {
        let mut buf = ryu::Buffer::new();
        let str = buf.format(v);
        self.push_str(str)
    }
}

macro_rules! serialize_num {
    ($($name:ident($t:ty) $parse:ident;)*) => {$(
        fn $name(self, v: $t) -> Result<Self::Ok> { self.$parse(v) }
    )*};
}

impl<T: Target, M: KeyValMode<T>> ser::Serializer for KeyValSerializer<'_, T, M> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let value = if v { "true" } else { "false" };
        self.push_str(value)
    }

    serialize_num! {
        serialize_i8(i8) push_int;
        serialize_i16(i16) push_int;
        serialize_i32(i32) push_int;
        serialize_i64(i64) push_int;
        serialize_i128(i128) push_int;
        serialize_u8(u8) push_int;
        serialize_u16(u16) push_int;
        serialize_u32(u32) push_int;
        serialize_u64(u64) push_int;
        serialize_u128(u128) push_int;
        serialize_f32(f32) push_float;
        serialize_f64(f64) push_float;
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.push_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.push_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.push(v)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        M::serialize_none(self.0)
    }

    fn serialize_some<U: ?Sized + ser::Serialize>(
        self,
        value: &U
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        M::serialize_unit(self.0)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Err(Error::UnsupportedKeyValType("struct"))
    }

    fn serialize_newtype_struct<U: ?Sized + ser::Serialize> (
        self,
        _name: &'static str,
        value: &U,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedKeyValType("tuple struct"))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        self.push_str(name)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedKeyValType("struct variant"))
    }
    
    fn serialize_newtype_variant<U: ?Sized + ser::Serialize> (
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &U,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedKeyValType("newtype variant"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedKeyValType("tuple variant"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.push_str(variant)
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedKeyValType("tuple"))
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedKeyValType("sequence"))
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedKeyValType("map"))
    }
}

pub trait KeyValMode<T: Target> {
    fn push(
        encoder: &mut Encoder<T>,
        value: &[u8]
    ) -> Result<()>;

    fn serialize_unit(encoder: &mut Encoder<T>) -> Result<()>;
    fn serialize_none(encoder: &mut Encoder<T>) -> Result<()>;
}

pub struct Key;
impl<T: Target> KeyValMode<T> for Key {
    fn push(
        encoder: &mut Encoder<T>,
        value: &[u8]
    ) -> Result<()> {
        encoder.push_key(value)
    }

    fn serialize_unit(_encoder: &mut Encoder<T>) -> Result<()> {
        Err(Error::UnsupportedKeyValType("unit"))
    }

    fn serialize_none(_encoder: &mut Encoder<T>) -> Result<()> {
        Err(Error::UnsupportedKeyValType("none"))
    }
}

pub struct Val;
impl<T: Target> KeyValMode<T> for Val {
    fn push(
        encoder: &mut Encoder<T>,
        value: &[u8]
    ) -> Result<()> {
        encoder.push_value(value)
    }

    fn serialize_unit(encoder: &mut Encoder<T>) -> Result<()> {
        encoder.push_empty_value()
    }

    fn serialize_none(encoder: &mut Encoder<T>) -> Result<()> {
        encoder.push_none_value()
    }
}
