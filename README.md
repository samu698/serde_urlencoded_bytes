# `serde_urlencoded_bytes`
Serde serializer and deserializer for the [`x-www-form-urlencoded`][1] format,
that supports non UTF-8 strings.

The `x-www-form-urlencoded` format allows to store a list of key-value pairs
where both the key and value are percent encoded strings, the value portion can
be fully omitted. Percent encoding is used to escape characters with special
meaning and allows to encode any string value.

The web standard requires that the decoded and encoded strings to be UTF-8
encoded, this crate drops that requirement allowing for a more flexible format
that can store raw seqences of bytes. The crate [serde_bytes][2] can be used to
construct values serialized as byte strings.

## Example
```rust
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Value {
    uint: u32,
    int: i32,
    str: String,
    #[serde(with = "serde_bytes")]
    bytes: Vec<u8>,
}

fn main() {
    let bytes = b"uint=10&int=-10&str=Hello%2c+World&bytes=%f0%0d%ba%be";
    let value = Value {
        uint: 10,
        int: -10,
        str: "Hello, World".into(),
        bytes: b"\xf0\x0d\xba\xbe".into(),
    };

    let encoded = serde_urlencoded_bytes::to_vec(&value).unwrap();
    assert_eq!(encoded, bytes);

    let decoded: Value = serde_urlencoded_bytes::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, value);
}
```

## Format
This crate aims to be as flexible as possible when mapping between Rust data
types and the underlying data format. `x-www-form-urlencoded` consists of a list
of key-value pairs, so all Rust representation must boil down to that structure.

### Top-Level Representation
The following Rust types can be used:
- **Structs** - Field names are used as keys
- **Maps** - Such as `HashMap<K, V>`
- **Sequences of pairs** - Such as `Vec<(K, V)>`
- **Options**:
  - `None` - Maps to the empty list
  - `Some(T)` - Maps to the inner top-level value
- **Newtype structs** - Wrap another supported top-level type
- **Unit structs** - Maps to the empty list
- **Unit `()`** - Maps to the empty list

### Pair Representation
Individual key-value pairs may be represented as:
- **2-element tuples** `(K, V)`
- **2-element sequences** - Such as `vec![key, value]`
- **2-element tuple struct** - Such as `Pair(K, V)`
- **Options**:
  - `None` - Omits the pair entirely
  - `Some(T)` - Maps to the inner pair
- **Newtype structs** - Wrap another supported pair type

### Key-Value Representation
Keys and values may be represented as:
- **Primitive types** - Any type with a string representation
- **Unit structs** - Maps to the struct name
- **Unit enum variants** - Maps to the variant name
- **Options**:
  - `None` - Omits the value entirely `*`
  - `Some(T)` - Maps to the inner value
- **Unit `()`** - produces a key without a value `*`
- **Newtype structs** — transparently wrap another supported value type.

Items marked with a `*` can only be used as value types.

[1]: <https://url.spec.whatwg.org/#application/x-www-form-urlencoded>
[2]: <https://docs.rs/serde_bytes/latest/serde_bytes>
