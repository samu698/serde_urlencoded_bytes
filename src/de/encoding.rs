use std::borrow::Cow;

pub struct Part<'a>(Cow<'a, [u8]>);

impl<'a> Part<'a> {
    pub fn empty() -> Self { Self(Cow::Borrowed(&[][..])) }

    pub fn decode(bytes: &'a [u8]) -> Self {
        fn from_hex(b: u8) -> Option<u8> {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            }
        }

        /// Search for the first replacement needed to decode the form 
        /// urlencoded bytes.
        ///
        /// If a replacement is found then a triple `(prefix, ch, rest)` is 
        /// returned where `prefix` are the unchanged bytes before the
        /// replacement, `ch` is the replacement byte and `rest` are the
        /// remaining bytes that might need more replacements.
        ///
        /// Returns `None` if no replacements are needed.
        fn find_replacement(bytes: &[u8]) -> Option<(&[u8], u8, &[u8])> {
            for (idx, &b) in bytes.iter().enumerate() {
                if b == b'+' {
                    return Some((&bytes[..idx], b' ', &bytes[idx + 1..]));
                }

                if b == b'%' 
                    && let Some(hi) = bytes.get(idx + 1).copied().and_then(from_hex)
                    && let Some(lo) = bytes.get(idx + 2).copied().and_then(from_hex) 
                {
                    let ch = 16 * hi + lo;
                    return Some((&bytes[..idx], ch, &bytes[idx + 3..]));
                }
            }

            None
        }

        let Some((head, ch, mut rest)) = find_replacement(bytes) else {
            return Self(Cow::Borrowed(bytes));
        };

        let mut result = head.to_vec();
        result.push(ch);

        while let Some((head, ch, tail)) = find_replacement(rest) {
            result.extend_from_slice(head);
            result.push(ch);
            rest = tail;
        }
        result.extend_from_slice(rest);

        Self(Cow::Owned(result))
    }

    pub fn value(&self) -> &Cow<'a, [u8]> { &self.0 }

    pub fn inner(self) -> Cow<'a, [u8]> { self.0 }
}

pub struct PairIter<'a>(pub &'a [u8]);

impl<'a> PairIter<'a> {
    fn read_sequence(&mut self) -> Option<&'a [u8]> {
        if self.0.is_empty() { return None }
        let (seq, rest) = match self.0.iter().position(|&b| b == b'&') {
            Some(i) => (&self.0[..i], &self.0[i + 1..]),
            None => (self.0, &[][..])
        };
        self.0 = rest;
        Some(seq)
    }
}

impl<'a> Iterator for PairIter<'a> {
    type Item = (Part<'a>, Part<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        let seq = loop {
            let sequence = self.read_sequence()?;
            if !sequence.is_empty() { break sequence; }
        };

        let pair = match seq.iter().position(|&b| b == b'=') {
            Some(i) => (Part::decode(&seq[..i]), Part::decode(&seq[i + 1..])),
            None => (Part::decode(seq), Part::empty())
        };

        Some(pair)
    }
}
