use std::mem::ManuallyDrop;

use super::{Error, Result};

pub trait Target {
    fn push(&mut self, ch: char);

    fn index(&self) -> usize;
    fn revert(&mut self, index: usize);
}

impl Target for String {
    fn push(&mut self, ch: char) { self.push(ch); }
    fn index(&self) -> usize { self.len() }
    fn revert(&mut self, index: usize) { self.truncate(index); }
}

impl Target for &mut String {
    fn push(&mut self, ch: char) { (*self).push(ch); }

    fn index(&self) -> usize { self.len() }
    fn revert(&mut self, index: usize) { self.truncate(index); }
}

impl Target for Vec<u8> {
    fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf).as_bytes();
        self.extend_from_slice(bytes);
    }

    fn index(&self) -> usize { self.len() }
    fn revert(&mut self, index: usize) { self.truncate(index); }
}

impl Target for &mut Vec<u8> {
    fn push(&mut self, ch: char) { 
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf).as_bytes();
        self.extend_from_slice(bytes);
    }

    fn index(&self) -> usize { self.len() }
    fn revert(&mut self, index: usize) { self.truncate(index); }
}

enum Expected { FirstKey, Key, Value(usize) }

pub struct Encoder<T: Target> {
    target: ManuallyDrop<T>,
    start_index: usize,
    expected: Expected,
}

impl<T: Target> Encoder<T> {
    pub fn new(target: T) -> Self {
        let target = ManuallyDrop::new(target);
        let start_index = target.index();
        Self { target, start_index, expected: Expected::FirstKey }
    }

    pub fn push_key(&mut self, key: &[u8]) -> Result<()> {
        let index = self.target.index();
        match std::mem::replace(&mut self.expected, Expected::Value(index)) {
            Expected::FirstKey => {}
            Expected::Key => self.target.push('&'),
            Expected::Value(_) => return Err(Error::ExpectedValue)
        }
        self.append_encoded(key);
        Ok(())
    }

    pub fn push_value(&mut self, value: &[u8]) -> Result<()> {
        match std::mem::replace(&mut self.expected, Expected::Key) {
            Expected::FirstKey | Expected::Key => return Err(Error::ExpectedKey),
            Expected::Value(_) => {}
        }
        self.target.push('=');
        self.append_encoded(value);
        Ok(())
    }

    pub fn push_none_value(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.expected, Expected::Key) {
            Expected::FirstKey | Expected::Key => return Err(Error::ExpectedKey),
            Expected::Value(index) => self.target.revert(index),
        }
        Ok(())
    }

    pub fn push_empty_value(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.expected, Expected::Key) {
            Expected::FirstKey | Expected::Key => return Err(Error::ExpectedKey),
            Expected::Value(_) => {}
        }
        Ok(())
    }

    pub fn finish(self) -> Result<T> {
        match self.expected {
            Expected::FirstKey | Expected::Key => {
                let mut slf = ManuallyDrop::new(self);
                // SAFETY: self.target will not be used after this call, because
                //         slf is wrapped in a ManuallyDrop preventing the call
                //         to Drop
                unsafe { Ok(ManuallyDrop::take(&mut slf.target)) }
            }
            Expected::Value(_) => Err(Error::ExpectedValue),
        }
    }

    fn append_encoded(&mut self, value: &[u8]) {
        for ch in value {
            match ch {
                b'-' | b'.' | b'0' ..= b'9' | b'A' ..= b'Z' | b'_' | b'a' ..= b'z' | b'~' => {
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

impl<T: Target> Drop for Encoder<T> {
    fn drop(&mut self) {
        self.target.revert(self.start_index);
        // SAFETY: self.target will not be used after this call, because self
        //         is being dropped
        unsafe { ManuallyDrop::drop(&mut self.target); }
    }
}
