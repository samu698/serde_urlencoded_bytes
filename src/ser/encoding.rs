use super::{Error, Result};

pub trait Target {
    fn push(&mut self, ch: char);
}

impl Target for String {
    fn push(&mut self, ch: char) { self.push(ch); }
}

impl Target for &mut String {
    fn push(&mut self, ch: char) { (*self).push(ch); }
}

impl Target for Vec<u8> {
    fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf).as_bytes();
        self.extend_from_slice(bytes);
    }
}

impl Target for &mut Vec<u8> {
    fn push(&mut self, ch: char) { 
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf).as_bytes();
        self.extend_from_slice(bytes);
    }
}

enum Expected { FirstKey, Key, Value }

pub struct Encoder<T: Target> {
    target: T,
    expected: Expected,
}

impl<T: Target> Encoder<T> {
    pub fn new(target: T) -> Self {
        Self { target, expected: Expected::FirstKey }
    }

    pub fn push_key(&mut self, key: &[u8]) -> Result<()> {
        match std::mem::replace(&mut self.expected, Expected::Value) {
            Expected::FirstKey => {}
            Expected::Key => self.target.push('&'),
            Expected::Value => return Err(Error::ExpectedValue)
        }
        self.append_encoded(key);
        Ok(())
    }

    pub fn push_value(&mut self, value: &[u8]) -> Result<()> {
        match std::mem::replace(&mut self.expected, Expected::Key) {
            Expected::FirstKey | Expected::Key => return Err(Error::ExpectedKey),
            Expected::Value => {}
        }
        self.target.push('=');
        self.append_encoded(value);
        Ok(())
    }

    pub fn push_empty_value(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.expected, Expected::Key) {
            Expected::FirstKey | Expected::Key => return Err(Error::ExpectedKey),
            Expected::Value => {}
        }
        Ok(())
    }

    pub fn finish(self) -> Result<T> {
        match self.expected {
            Expected::FirstKey | Expected::Key => Ok(self.target),
            Expected::Value => return Err(Error::ExpectedValue)
        }
    }

    fn append_encoded(&mut self, value: &[u8]) {
        for ch in value {
            match ch {
                b'*' | b'-' | b'.' | b'0' ..= b'9' | b'A' ..= b'Z' | b'_' | b'a' ..= b'z' => {
                    self.target.push(*ch as char);
                }
                b' ' => self.target.push('+'),
                ch => {
                    let (hi, lo) = (ch / 16, ch % 16);
                    let hi = if hi < 10 { hi + b'0' } else { hi + b'a' - 10 };
                    let lo = if lo < 10 { lo + b'0' } else { lo + b'a' - 10 };
                    self.target.push('%');
                    self.target.push(hi as char);
                    self.target.push(lo as char);
                }
            }
        }
    }
}

