use crate::bytesx::buf_vec_deque::BufVecDeque;
use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use httpbis::BufGetBytes;
use std::collections::vec_deque;
use std::collections::VecDeque;
use std::io::IoSlice;

#[derive(Debug, Default)]
pub(crate) struct BytesVecDeque {
    deque: BufVecDeque<Bytes>,
}

impl<I: Into<VecDeque<Bytes>>> From<I> for BytesVecDeque {
    fn from(deque: I) -> Self {
        BytesVecDeque {
            deque: deque.into().into(),
        }
    }
}

impl Into<Bytes> for BytesVecDeque {
    fn into(self) -> Bytes {
        self.into_bytes()
    }
}

impl Into<Vec<u8>> for BytesVecDeque {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.remaining());
        for b in self.deque {
            v.extend_from_slice(b.as_ref());
        }
        v
    }
}

impl BytesVecDeque {
    pub fn _new() -> BytesVecDeque {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.deque.remaining()
    }

    pub fn extend(&mut self, bytes: Bytes) {
        if bytes.is_empty() {
            return;
        }
        self.deque.push_back(bytes);
    }

    pub fn get_bytes(&self) -> Bytes {
        let mut bytes_mut = BytesMut::with_capacity(self.remaining());
        for b in &self.deque {
            bytes_mut.extend_from_slice(b);
        }
        bytes_mut.freeze()
    }

    pub fn into_bytes(mut self) -> Bytes {
        self.to_bytes()
    }
}

impl Buf for BytesVecDeque {
    fn remaining(&self) -> usize {
        self.deque.remaining()
    }

    fn chunk(&self) -> &[u8] {
        self.deque.chunk()
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        self.deque.chunks_vectored(dst)
    }

    fn advance(&mut self, cnt: usize) {
        self.deque.advance(cnt)
    }

    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        self.deque.copy_to_bytes(len)
    }
}

impl BufGetBytes for BytesVecDeque {
    fn get_bytes(&mut self, cnt: usize) -> Bytes {
        self.deque.get_bytes(cnt)
    }
}

impl<'a> IntoIterator for &'a BytesVecDeque {
    type Item = &'a Bytes;
    type IntoIter = vec_deque::Iter<'a, Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.deque).into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buf_empty() {
        let d = BytesVecDeque::default();
        assert_eq!(&[0u8; 0], Buf::chunk(&d));
        assert_eq!(0, Buf::remaining(&d));
        assert_eq!(false, Buf::has_remaining(&d));
    }

    #[test]
    fn buf_advance_full() {
        let mut d = BytesVecDeque::default();
        d.extend(Bytes::from_static(b"ab"));
        d.extend(Bytes::from_static(b"cde"));

        assert_eq!(b"ab", Buf::chunk(&d));
        Buf::advance(&mut d, 2);
        assert_eq!(b"cde", Buf::chunk(&d));
        Buf::advance(&mut d, 3);
        assert_eq!(0, Buf::remaining(&d));
        assert_eq!(false, Buf::has_remaining(&d));
    }

    #[test]
    fn buf_advance() {
        let mut d = BytesVecDeque::default();
        d.extend(Bytes::from_static(b"ab"));
        d.extend(Bytes::from_static(b"cde"));

        assert_eq!(b"ab", Buf::chunk(&d));
        Buf::advance(&mut d, 1);
        assert_eq!(b"b", Buf::chunk(&d));
        Buf::advance(&mut d, 3);
        assert_eq!(b"e", Buf::chunk(&d));
        Buf::advance(&mut d, 1);
        assert_eq!(0, Buf::remaining(&d));
        assert_eq!(false, Buf::has_remaining(&d));
    }

    #[test]
    fn buf_bytes_vectored() {
        let mut d = BytesVecDeque::default();
        d.extend(Bytes::from_static(b"ab"));
        d.extend(Bytes::from_static(b"cde"));

        let mut v = [IoSlice::new(&[])];
        assert_eq!(1, d.chunks_vectored(&mut v));
        assert_eq!(b"ab", &*v[0]);

        let mut v = [IoSlice::new(&[]), IoSlice::new(&[])];
        assert_eq!(2, d.chunks_vectored(&mut v));
        assert_eq!(b"ab", &*v[0]);
        assert_eq!(b"cde", &*v[1]);

        let mut v = [IoSlice::new(&[]), IoSlice::new(&[]), IoSlice::new(&[])];
        assert_eq!(2, d.chunks_vectored(&mut v));
        assert_eq!(b"ab", &*v[0]);
        assert_eq!(b"cde", &*v[1]);
    }
}
