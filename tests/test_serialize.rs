use std::fmt::Display;
use std::collections::BTreeMap;

use serde::{Serialize, Serializer, ser::SerializeSeq};
use serde_derive::Serialize;
use serde_urlencoded_bytes as sut;

#[derive(Debug, Serialize, PartialEq)]
struct Bytes(#[serde(with = "serde_bytes")] &'static [u8]);
fn b<const N: usize>(v: &'static [u8; N]) -> Bytes { Bytes(&v[..]) }

#[derive(Debug, Serialize, PartialEq)]
struct Struct<T> { a: T, b: T }

#[derive(Debug, Serialize, PartialEq)]
struct Newtype<T>(T);

#[derive(Debug, Serialize, PartialEq)]
struct Tuple<T>(T, T);

#[derive(Debug, Serialize, PartialEq)]
struct Unit;

#[derive(Debug, PartialEq)]
struct UnsizedSeq<T>(Vec<T>);
impl<T: Serialize> Serialize for UnsizedSeq<T> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut ser = ser.serialize_seq(None)?;
        self.0.iter().try_for_each(|elem| ser.serialize_element(&elem))?;
        ser.end()
    }
}

macro_rules! check {
    ($input:expr, $output:expr) => {
        assert_eq!(
            sut::to_string(&($input)).as_deref(),
            Ok($output)
        )
    };
}

macro_rules! check_err {
    ($input:expr) => { assert!(sut::to_string(&($input)).is_err()) };
}

struct RyuFloat<T>(T);
impl<T: ryu::Float> Display for RyuFloat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut buffer = ryu::Buffer::new();
        write!(f, "{}", buffer.format(self.0))
    }
}

#[test]
fn append_string() {
    let mut v = "->".to_string();
    assert!(sut::append_string(&mut v, &[(0, 0)]).is_ok());
    assert_eq!(v, "->0=0");

    assert!(sut::append_string(&mut v, "don't write").is_err());
    assert!(sut::append_string(&mut v, &[("don't write", Tuple(0, 0))]).is_err());
    assert_eq!(v, "->0=0");
}

#[test]
fn to_vec() {
    assert_eq!(sut::to_vec(&[(0, Some(0)), (1, None)]), Ok(b"0=0".to_vec()));
}

#[test]
fn append_vec() {
    let mut v = b"->".to_vec();
    assert!(sut::append_vec(&mut v, &[(0, 0)]).is_ok());
    assert_eq!(v, b"->0=0");

    assert!(sut::append_vec(&mut v, &false).is_err());
    assert!(sut::append_vec(&mut v, &[("don't write", Tuple(0, 0))]).is_err());
    assert_eq!(v, b"->0=0");
}

#[test]
fn serialize_toplevel() {
    // Empty array
    let empty_arr: [(); 0] = [];
    check!(empty_arr, "");

    // Empty Sequence
    let empty_vec: Vec<()> = vec![];
    check!(empty_vec, "");

    // Sized Sequence
    check!(vec![(0, 0)], "0=0");

    // Unsized Sequence
    check!(UnsizedSeq(vec![(0, 0)]), "0=0");

    // Unit
    check!((), "");

    // Unit struct
    check!(Unit, "");

    // Options
    check!(None::<u8>, "");
    check!(Some([(0, 0)]), "0=0");

    // Struct
    check!(Struct { a: "a", b: "b" }, "a=a&b=b");

    // Newtype struct of pairs
    check!(Newtype([(0, 0)]), "0=0");

    // Map
    let mut map = BTreeMap::new();
    map.insert("a", 0);
    map.insert("b", 1);
    check!(map, "a=0&b=1");
}

#[test]
fn serialize_pair() {
    // Invalid length tuple
    check_err!([(0)]);
    check_err!([(0, 0, 0)]);

    // Optional pairs
    check!([Some((0, 0)), None], "0=0");

    // Sized sequences
    check!([vec![0, 0]], "0=0");
    check_err!([vec![0]]);
    check_err!([vec![0, 0, 0]]);

    // Unsized sequences
    check!([UnsizedSeq(vec![0, 0])], "0=0");
    check_err!([UnsizedSeq(vec![0])]);
    check_err!([UnsizedSeq(vec![0, 0, 0])]);

    // Newtype struct of pair
    check!([Newtype((0, 0))], "0=0");

    // Tuple struct of length 2
    check!([Tuple(0, 0)], "0=0");
}

#[test]
fn serialize_bool() {
    check!([("true", true), ("false", false)], "true=true&false=false");
    check!([(true, false), (true, false)], "true=false&true=false");
}

macro_rules! serialize_num {
    ($($name:ident($t:ty$(, $wrap:expr)?);)*) => {$(
        #[test]
        fn $name() {
            let (min, max) = (<$t>::MIN, <$t>::MAX);

            let v = [("min", min), ("max", max)];
            let s = format!(
                "min={min}&max={max}",
                $(min = $wrap(min), max = $wrap(max))?
            );
            check!(v, s.as_str());

            let v = [(max, min), (min, max)];
            let s = format!(
                "{max}={min}&{min}={max}",
                $(min = $wrap(min), max = $wrap(max))?
            );
            check!(v, s.as_str());
        }
    )*};
}

serialize_num! {
    serialize_i8(i8);
    serialize_i16(i16);
    serialize_i32(i32);
    serialize_i64(i64);
    serialize_i128(i128);
    serialize_u8(u8);
    serialize_u16(u16);
    serialize_u32(u32);
    serialize_u64(u64);
    serialize_u128(u128);
    serialize_f32(f32, RyuFloat);
    serialize_f64(f64, RyuFloat);
}

#[test]
fn serialize_char() {
    check!([('x', 'y')], "x=y");
    check!([('=', '&')], "%3d=%26");
    check!([(' ', '🔥')], "+=%f0%9f%94%a5");
}

#[test]
fn serialize_str() {
    check!([("abc", "xyz"), ("abc", ""), ("", "xyz")], "abc=xyz&abc=&=xyz");
    check!(
        [(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~",
            "!*'();:@&=+$,/?%#[] "
        )],
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~=\
        %21%2a%27%28%29%3b%3a%40%26%3d%2b%24%2c%2f%3f%25%23%5b%5d+"
    );
}

#[test]
fn serialize_bytes() {
    check!(
        [(b(b"\xf1\xf2\xf3\xf4"), b(b"\xea\xeb\xec\xed"))],
        "%f1%f2%f3%f4=%ea%eb%ec%ed"
    );
}

#[test]
fn serialize_option() {
    check!([("some", Some(0)), ("none", None)], "some=0");
    check!([(Some("some"), Some(0)), (Some("none"), None)], "some=0");
    check_err!([(None::<u8>, 0)]);
}

#[test]
fn serialize_unit() {
    check!([("null", ()), ("void", ())], "null&void");
    check_err!([((), "error")]);
}

#[test]
fn serialize_struct() {
    check_err!([("v", Struct { a: 0, b: 0 })]);

    check!([("null", Unit), ("void", Unit)], "null=Unit&void=Unit");
    check!([(Unit, ())], "Unit");

    check!([("int", Newtype(33))], "int=33");
    check!([("str", Newtype("hello"))], "str=hello");

    check_err!([("v", Tuple(0, 0))]);
}

#[test]
fn serialize_enum() {
    #[derive(Debug, Serialize, PartialEq)]
    enum Enum {
        UnitA,
        UnitB,
        Struct { a: u8 },
        Newtype(u8),
        Tuple(u8, u8),
    }

    check!([("a", Enum::UnitA), ("b", Enum::UnitB)], "a=UnitA&b=UnitB");
    check_err!([("v", Enum::Struct { a: 0 })]);
    check_err!([("v", Enum::Newtype(0))]);
    check_err!([("v", Enum::Tuple(0, 0))]);
}
