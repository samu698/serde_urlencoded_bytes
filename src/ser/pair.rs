use serde::ser;

use super::encoding::{Target, Encoder};
use super::keyval::{KeySerializer, ValSerializer};
use super::{Result, Error, SerErr, SerOk};

enum State { Key, Value, Done }
pub struct PairSerializer<'a, T: Target> {
    encoder: &'a mut Encoder<T>,
    state: State,
}

macro_rules! invalid_pair {
    ($($name:ident$(<$T:ident>)?($($arg:ty),*) -> $ret:ident;)*) => {$(
        fn $name $(<$T: ?Sized + ser::Serialize>)? (
            self,
            $(_: $arg),*
        ) -> Result<Self::$ret> {
            Err(Error::InvalidPair)
        }
    )*};
}

impl<'a, T: Target> PairSerializer<'a, T> {
    pub fn new(encoder: &'a mut Encoder<T>) -> Self {
        Self { encoder, state: State::Key }
    }

    fn serialize_element<E: ?Sized + ser::Serialize>(
        &mut self,
        value: &E
    ) -> Result<()> {
        let (result, state) = match self.state {
            State::Key => (value.serialize(KeySerializer::new(self.encoder)), State::Value),
            State::Value => (value.serialize(ValSerializer::new(self.encoder)), State::Done),
            State::Done => unreachable!("Pair has length greater than two"),
        };
        self.state = state;
        result
    }

    fn end(self) -> Result<SerOk<Self>> {
        match self.state {
            State::Done => Ok(()),
            _ => unreachable!("Pair has length smaller than two"),
        }
    }
}

impl<'a, T: Target> ser::Serializer for PairSerializer<'a, T> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    invalid_pair! {
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
        serialize_unit() -> Ok;
        serialize_unit_struct(&'static str) -> Ok;
        serialize_unit_variant(&'static str, u32, &'static str) -> Ok;
        serialize_newtype_variant<U>(&'static str, u32, &'static str, &U) -> Ok;
        serialize_tuple_struct(&'static str, usize) -> SerializeTupleStruct;
        serialize_tuple_variant(&'static str, u32, &'static str, usize) -> SerializeTupleVariant;
        serialize_map(Option<usize>) -> SerializeMap;
        serialize_struct(&'static str, usize) -> SerializeStruct;
        serialize_struct_variant(&'static str, u32, &'static str, usize) -> SerializeStructVariant;
    }

    fn serialize_newtype_struct<U: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &U,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<U: ?Sized + ser::Serialize>(
        self,
        value: &U,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeSeq> {
        if len.is_some_and(|len| len == 2) {
            Ok(self)
        } else {
            Err(Error::InvalidPairLen)
        }
    }

    fn serialize_tuple(
        self,
        len: usize
    ) -> Result<Self::SerializeTuple> {
        if len == 2 {
            Ok(self)
        } else {
            Err(Error::InvalidPairLen)
        }
    }
}

impl<'a, T: Target> ser::SerializeSeq for PairSerializer<'a, T> {
    type Ok = SerOk<Self>;
    type Error = SerErr<Self>;

    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U
    ) -> Result<()> {
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end()
    }
}

impl<'a, T: Target> ser::SerializeTuple for PairSerializer<'a, T> {
    type Ok = SerOk<Self>;
    type Error = SerErr<Self>;

    fn serialize_element<U: ?Sized + ser::Serialize>(
        &mut self,
        value: &U
    ) -> Result<()> {
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end()
    }
}
