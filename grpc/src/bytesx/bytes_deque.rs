use crate::bytesx::buf_eq::buf_eq;
use crate::bytesx::bytes_vec_deque::BytesVecDeque;
use crate::bytesx::iter_buf::IterBuf;
use bytes::Buf;
use bytes::Bytes;
use httpbis::BufGetBytes;
use std::collections::vec_deque;
use std::io::IoSlice;
use std::mem;

#[derive(Debug)]
enum Inner {
    One(Bytes),
    Deque(BytesVecDeque),
}

impl Default for Inner {
    fn default() -> Self {
        Inner::One(Bytes::new())
    }
}

/// `VecDeque<Bytes>` but slightly more efficient.
#[derive(Debug, Default)]
pub struct BytesDeque(Inner);

impl BytesDeque {
    /// Empty deque.
    pub fn new() -> BytesDeque {
        Default::default()
    }

    /// Create a deque by copying from bytes slice.
    pub fn copy_from_slice(bytes: &[u8]) -> BytesDeque {
        BytesDeque::from(Bytes::copy_from_slice(bytes))
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        match &self.0 {
            Inner::One(b) => b.len(),
            Inner::Deque(d) => d.len(),
        }
    }

    /// Append [`Bytes`] to this deque.
    pub fn extend(&mut self, bytes: Bytes) {
        if bytes.is_empty() {
            return;
        }

        match &mut self.0 {
            Inner::One(one) if one.is_empty() => {
                self.0 = Inner::One(bytes);
            }
            Inner::One(one) => {
                self.0 = Inner::Deque(BytesVecDeque::from(vec![mem::take(one), bytes]));
            }
            Inner::Deque(deque) if deque.len() == 0 => {
                self.0 = Inner::One(bytes);
            }
            Inner::Deque(deque) => {
                deque.extend(bytes);
            }
        }
    }

    /// Get deque contents as [`Bytes`] object.
    ///
    /// This operation is cheap if this deque contains only single [`Bytes`] object,
    /// otherwise it allocates memory and copies data.
    pub fn get_bytes(&self) -> Bytes {
        match &self.0 {
            Inner::One(b) => b.clone(),
            Inner::Deque(d) => d.get_bytes(),
        }
    }

    /// Convert contents into [`Bytes`] object.
    ///
    /// This function tries to avoid memory allocation when possible.
    pub fn into_bytes(self) -> Bytes {
        match self.0 {
            Inner::One(b) => b,
            Inner::Deque(d) => d.into_bytes(),
        }
    }

    fn iter_buf<'a>(&'a self) -> IterBuf<Bytes, impl Iterator<Item = Bytes> + 'a> {
        IterBuf::new(self.into_iter().cloned(), self.remaining())
    }
}

impl PartialEq<BytesDeque> for BytesDeque {
    fn eq(&self, other: &BytesDeque) -> bool {
        buf_eq(self.iter_buf(), other.iter_buf())
    }
}

impl PartialEq<[u8]> for BytesDeque {
    fn eq(&self, other: &[u8]) -> bool {
        buf_eq(self.iter_buf(), other)
    }
}

impl From<Bytes> for BytesDeque {
    fn from(b: Bytes) -> Self {
        BytesDeque(Inner::One(b))
    }
}

impl From<Vec<u8>> for BytesDeque {
    fn from(v: Vec<u8>) -> Self {
        BytesDeque::from(Bytes::from(v))
    }
}

impl<'a> From<&'a str> for BytesDeque {
    fn from(s: &'a str) -> Self {
        BytesDeque::from(Bytes::copy_from_slice(s.as_bytes()))
    }
}

impl Into<Bytes> for BytesDeque {
    fn into(self) -> Bytes {
        self.into_bytes()
    }
}

impl Into<Vec<u8>> for BytesDeque {
    fn into(self) -> Vec<u8> {
        match self.0 {
            Inner::One(b) => Vec::from(b.as_ref()),
            Inner::Deque(d) => d.into(),
        }
    }
}

impl Buf for BytesDeque {
    fn remaining(&self) -> usize {
        match &self.0 {
            Inner::One(b) => b.remaining(),
            Inner::Deque(d) => d.remaining(),
        }
    }

    fn chunk(&self) -> &[u8] {
        match &self.0 {
            Inner::One(b) => b.chunk(),
            Inner::Deque(d) => d.chunk(),
        }
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        match &self.0 {
            Inner::One(b) => b.chunks_vectored(dst),
            Inner::Deque(d) => d.chunks_vectored(dst),
        }
    }

    fn advance(&mut self, cnt: usize) {
        match &mut self.0 {
            Inner::One(b) => b.advance(cnt),
            Inner::Deque(d) => d.advance(cnt),
        }
    }
}

impl BufGetBytes for BytesDeque {
    fn get_bytes(&mut self, cnt: usize) -> Bytes {
        match &mut self.0 {
            Inner::One(b) => b.get_bytes(cnt),
            Inner::Deque(d) => d.get_bytes(cnt),
        }
    }
}

pub enum Iter<'a> {
    One(Option<&'a Bytes>),
    Deque(vec_deque::Iter<'a, Bytes>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::One(b) => b.take(),
            Iter::Deque(d) => d.next(),
        }
    }
}

impl<'a> IntoIterator for &'a BytesDeque {
    type Item = &'a Bytes;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match &self.0 {
            Inner::One(b) => Iter::One(Some(b)),
            Inner::Deque(d) => Iter::Deque((&d).into_iter()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::thread_rng;
    use rand::Rng;

    fn extend_iter() {
        let mut d = BytesDeque::new();
        let mut reference = Vec::new();

        for _ in 0..10 {
            let bytes = if thread_rng().gen_range(0..3) == 0 {
                Bytes::new()
            } else {
                let len = thread_rng().gen_range(0..10);
                let mut v = Vec::new();
                for _ in 0..len {
                    v.push(thread_rng().gen());
                }
                Bytes::from(v)
            };

            reference.extend_from_slice(&bytes);
            d.extend(bytes);
        }

        assert_eq!(reference, Into::<Vec<u8>>::into(d));
    }

    #[test]
    fn extend() {
        for _ in 0..10000 {
            extend_iter();
        }
    }
}
