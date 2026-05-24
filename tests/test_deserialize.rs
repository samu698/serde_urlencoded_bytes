use serde_derive::Deserialize;
use serde_urlencoded_bytes as sut;

macro_rules! check {
    ($input:expr, $output:expr) => {
        assert_eq!(sut::from_bytes($input), Ok($output))
    };
}

macro_rules! check_str {
    ($input:expr, $output:expr) => {
        assert_eq!(sut::from_str($input), Ok($output))
    };
}

#[derive(Deserialize, Debug, PartialEq)]
struct OwnedBytes(#[serde(with = "serde_bytes")] Vec<u8>);

#[derive(Deserialize, Debug, PartialEq)]
struct Unit;

#[derive(Deserialize, Debug, PartialEq)]
struct Newtype<A>(A);

#[derive(Deserialize, Debug, PartialEq)]
struct Tuple<A, B>(A, B);

#[derive(Deserialize, Debug, PartialEq)]
struct Struct<A, B> { a: A, b: B }

type MapSame<A> = Struct<A, A>;

fn s(v: &str) -> String { v.to_string() }
fn b(v: &[u8]) -> OwnedBytes { OwnedBytes(v.to_vec()) }

#[test]
fn deserialize_toplevel() {
    // Empty Sequence
    check!(b"", Vec::<()>::new());

    // Sequence
    check!(b"0=0", vec![(0, 0)]);

    // Unit
    check!(b"", ());

    // Unit struct
    check!(b"", Unit);

    // Options
    check!(b"", None::<()>);
    check!(b"0=0", Some([(0, 0)]));

    // Struct
    check!(b"a=0&b=1", Struct { a: 0, b: 1 });

    // Newtype struct
    check!(b"0=0", Newtype([(0, 0)]));
}

#[test]
fn deserialize_bool() {
    let (t, f) = (true, false);

    let s = format!("true={t}&false={f}");
    let r = [("true", t), ("false", f)];
    check_str!(&s, r);

    let s = format!("{t}={f}&{f}={t}");
    let r = [(t, f), (f, t)];
    check_str!(&s, r);
}

macro_rules! deserialize_num {
    ($($name:ident($t:ty);)*) => {$(
        #[test]
        fn $name() {
            let (min, max) = (<$t>::MIN, <$t>::MAX);

            let s = format!("min={min}&max={max}");
            let r = [("min", min), ("max", max)];
            check_str!(&s, r);

            let s = format!("{max}={min}&{min}={max}");
            let r = [(max, min), (min, max)];
            check_str!(&s, r);
        }
    )*};
}

deserialize_num! {
    deserialize_i8(i8);
    deserialize_i16(i16);
    deserialize_i32(i32);
    deserialize_i64(i64);
    deserialize_i128(i128);
    deserialize_u8(u8);
    deserialize_u16(u16);
    deserialize_u32(u32);
    deserialize_u64(u64);
    deserialize_u128(u128);
    deserialize_f32(f32);
    deserialize_f64(f64);
}

#[test]
fn deserialize_char() {
    check!(b"char=x", [("char", 'x')]);
    check!(b"x=char", [('x', "char")]);
    check!(b"%26=+", [('&', ' ')]);
    check!(b"fire=%F0%9F%94%A5", [("fire", '🔥')]);

    assert!(sut::from_bytes::<[(&str, char); 1]>(b"invalid=%f0").is_err());
    assert!(sut::from_bytes::<[(&str, char); 1]>(b"len=abc").is_err());
    assert!(sut::from_bytes::<[(&str, char); 1]>(b"len=").is_err());
}

#[test]
fn deserialize_str() {
    check!(b"abc=xyz", [("abc", "xyz")]);
    check!(b"abc=", [("abc", "")]);
    check!(b"=xyz", [("", "xyz")]);
    check!(b"v=====", [("v", "====")]);

    assert!(sut::from_bytes::<[(&str, &str); 1]>(b"can+t=clon+e").is_err());
}

#[test]
fn deserialize_string() {
    check!(b"abc=xyz", [(s("abc"), s("xyz"))]);
    check!(b"abc=", [(s("abc"), s(""))]);
    check!(b"=xyz", [(s(""), s("xyz"))]);

    check!(b"%20%26+=%00%01", [(s(" & "), s("\0\x01"))]);
    check!(b"%0%58%0=%%58%", [(s("%0X%0"), s("%X%"))]);
    check!(b"%+%0+=%5+%58", [(s("% %0 "), s("%5 X"))]);

    assert!(sut::from_bytes::<[(String, String); 1]>(b"%F0=%FB").is_err());
}

#[test]
fn deserialize_bytes() {
    check!(b"abc=xyz", [(&b"abc"[..], &b"xyz"[..])]);
    check!(b"=xyz", [(&b""[..], &b"xyz"[..])]);
    check!(b"abc=", [(&b"abc"[..], &b""[..])]);

    assert!(sut::from_bytes::<[(&[u8], &[u8]); 1]>(b"can+t=clon+e").is_err());
}

#[test]
fn deserialize_byte_buf() {
    check!(b"abc=xyz", [(b(b"abc"), b(b"xyz"))]);
    check!(b"=xyz", [(b(b""), b(b"xyz"))]);
    check!(b"abc=", [(b(b"abc"), b(b""))]);

    check!(b"%f0%F1%f2%F3=%ea%Eb%eC%ED", [(b(b"\xF0\xF1\xF2\xF3"), b(b"\xEA\xEB\xEC\xED"))]);
}

#[test]
fn deserialize_option() {
    check!(b"a=a&b=b", [("a", Some("a")), ("b", Some("b"))]);
    check!(b"a=a", MapSame { a: Some("a"), b: None });
    check!(b"b=b", MapSame { a: None, b: Some("b") });
    check!(b"", MapSame::<Option<&str>> { a: None, b: None });
}

#[test]
fn deserialize_unit() {
    check!(b"", ());
    check!(b"&", ());
    check!(b"&&", ());

    assert!(sut::from_bytes::<()>(b"value=10").is_err());
    assert!(sut::from_bytes::<()>(b"value").is_err());
}

#[test]
fn deserialize_unit_struct() {
    check!(b"", Unit);
    check!(b"&", Unit);
    check!(b"&&", Unit);
}

#[test]
fn deserialize_newtype_struct() {
    check!(b"a=b", [(Newtype("a"), Newtype("b"))]);
    check!(b"true=0", [(Newtype(true), Newtype(0))]);
}

#[test]
fn deserialize_tuple_struct() {
    check!(b"a=b", [Tuple("a", "b")]);

    #[derive(Deserialize, Debug, PartialEq)]
    struct Triple<T>(T, T, T);
    assert!(sut::from_bytes::<Triple<&str>>(b"").is_err());
}

#[test]
fn deserialize_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Enum {
        UnitA,
        UnitB,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { a: u8, b: u8 },
    }

    check!(b"UnitA=UnitB", [(Enum::UnitA, Enum::UnitB)]);
    assert!(sut::from_bytes::<[(&str, Enum); 1]>(b"x=Newtype").is_err());
    assert!(sut::from_bytes::<[(&str, Enum); 1]>(b"x=Tuple").is_err());
    assert!(sut::from_bytes::<[(&str, Enum); 1]>(b"x=Struct").is_err());
}
