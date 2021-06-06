use std::cmp;
use std::collections::vec_deque;
use std::collections::VecDeque;
use std::io::IoSlice;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use bytes::Buf;
use bytes::BufMut;
use bytes::Bytes;
use bytes::BytesMut;
use httpbis::BufGetBytes;

#[derive(Debug)]
pub(crate) struct BufVecDeque<B: Buf> {
    deque: VecDeque<B>,
    len: usize,
}

impl<B: Buf> Default for BufVecDeque<B> {
    fn default() -> Self {
        BufVecDeque {
            deque: VecDeque::default(),
            len: 0,
        }
    }
}

impl<B: Buf, I: Into<VecDeque<B>>> From<I> for BufVecDeque<B> {
    fn from(deque: I) -> Self {
        let deque = deque.into();
        let len = deque.iter().map(Buf::remaining).sum();
        BufVecDeque { deque, len }
    }
}

impl<B: Buf> BufVecDeque<B> {
    pub fn _new() -> BufVecDeque<B> {
        Default::default()
    }

    pub fn _len(&self) -> usize {
        self.len
    }

    pub fn push_back(&mut self, bytes: B) {
        self.len += bytes.remaining();
        self.deque.push_back(bytes);
    }

    pub fn pop_back(&mut self) -> Option<B> {
        match self.deque.pop_back() {
            Some(b) => {
                self.len -= b.remaining();
                Some(b)
            }
            None => None,
        }
    }

    #[cfg(test)]
    pub fn back_mut(&mut self) -> Option<BufVecDequeBackMut<B>> {
        match self.deque.pop_back() {
            Some(back) => Some(BufVecDequeBackMut {
                deque: self,
                remaining: back.remaining(),
                back: Some(back),
            }),
            None => None,
        }
    }
}

impl<B: Buf> Buf for BufVecDeque<B> {
    fn remaining(&self) -> usize {
        self.len
    }

    fn chunk(&self) -> &[u8] {
        for b in &self.deque {
            let bytes = b.chunk();
            if !bytes.is_empty() {
                return &bytes;
            }
        }
        &[]
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        let mut n = 0;
        for b in &self.deque {
            if n == dst.len() {
                break;
            }
            n += b.chunks_vectored(&mut dst[n..]);
        }
        n
    }

    fn advance(&mut self, mut cnt: usize) {
        assert!(self.len >= cnt);
        self.len -= cnt;

        while cnt != 0 {
            let front = self.deque.front_mut().unwrap();
            let front_remaining = front.remaining();
            if cnt < front_remaining {
                front.advance(cnt);
                break;
            }

            self.deque.pop_front().unwrap();

            cnt -= front_remaining;
        }
    }

    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        todo!()
        // if !self.has_remaining() {
        //     Bytes::new()
        // } else if self.deque.len() == 1 {
        //     mem::take(self).into_iter().next().unwrap().to_bytes()
        // } else {
        //     let mut ret = BytesMut::with_capacity(self.remaining());
        //     ret.put(self);
        //     ret.freeze()
        // }
    }
}

impl<B: BufGetBytes> BufGetBytes for BufVecDeque<B> {
    fn get_bytes(&mut self, cnt: usize) -> Bytes {
        if cnt == 0 {
            return Bytes::new();
        }

        assert!(cnt <= self.remaining());

        match self.deque.front_mut() {
            Some(front) if front.remaining() >= cnt => {
                let r = if front.remaining() == cnt {
                    let mut front = self.deque.pop_front().unwrap();
                    front.get_bytes(cnt)
                } else {
                    front.get_bytes(cnt)
                };

                self.len -= cnt;
                return r;
            }
            _ => {}
        }

        let mut r = BytesMut::new();
        while r.len() < cnt {
            let chunk = self.chunk();
            debug_assert!(chunk.len() > 0);
            let copy = cmp::min(cnt - r.len(), chunk.len());
            r.extend_from_slice(&chunk[..copy]);
            self.advance(copy);
        }
        r.freeze()
    }
}

impl<'a, B: Buf> IntoIterator for &'a BufVecDeque<B> {
    type Item = &'a B;
    type IntoIter = vec_deque::Iter<'a, B>;

    fn into_iter(self) -> Self::IntoIter {
        self.deque.iter()
    }
}

impl<B: Buf> IntoIterator for BufVecDeque<B> {
    type Item = B;
    type IntoIter = vec_deque::IntoIter<B>;

    fn into_iter(self) -> Self::IntoIter {
        self.deque.into_iter()
    }
}

pub struct BufVecDequeBackMut<'a, B: Buf> {
    deque: &'a mut BufVecDeque<B>,
    back: Option<B>,
    remaining: usize,
}

impl<'a, B: Buf> Deref for BufVecDequeBackMut<'a, B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        self.back.as_ref().unwrap()
    }
}

impl<'a, B: Buf> DerefMut for BufVecDequeBackMut<'a, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.back.as_mut().unwrap()
    }
}

impl<'a, B: Buf> Drop for BufVecDequeBackMut<'a, B> {
    fn drop(&mut self) {
        let back = mem::take(&mut self.back).unwrap();
        let new_remaining = back.remaining();
        if new_remaining > self.remaining {
            self.deque.len += new_remaining - self.remaining;
        } else {
            self.deque.len -= self.remaining - new_remaining;
        }
        self.deque.deque.push_back(back);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn back_mut() {
        let mut d = BufVecDeque::<VecDeque<u8>>::default();
        d.push_back(VecDeque::from(vec![3, 4]));
        d.push_back(VecDeque::from(vec![4, 6]));
        assert_eq!(4, d.remaining());
        d.back_mut().unwrap().push_back(7);
        assert_eq!(5, d.remaining());
        d.back_mut().unwrap().pop_back();
        assert_eq!(4, d.remaining());
        d.back_mut().unwrap().pop_back();
        d.back_mut().unwrap().pop_back();
        assert_eq!(2, d.remaining());
    }

    #[test]
    fn pop_back() {
        let mut d = BufVecDeque::<VecDeque<u8>>::default();
        d.push_back(VecDeque::from(vec![3, 4]));
        d.push_back(VecDeque::from(vec![4, 6, 7]));

        d.pop_back().unwrap();
        assert_eq!(2, d.remaining());
    }

    #[test]
    fn get_bytes() {
        let mut d = BufVecDeque::from(vec![
            Bytes::copy_from_slice(b"ab"),
            Bytes::copy_from_slice(b"cde"),
        ]);

        assert_eq!(Bytes::copy_from_slice(b"a"), d.get_bytes(1));
        assert_eq!(4, d.remaining());
        assert_eq!(Bytes::copy_from_slice(b"b"), d.get_bytes(1));
        assert_eq!(3, d.remaining());
        assert_eq!(Bytes::copy_from_slice(b"cde"), d.get_bytes(3));
        assert_eq!(0, d.remaining());
    }

    #[test]
    fn get_bytes_cross_boundary() {
        let mut d = BufVecDeque::from(vec![
            Bytes::copy_from_slice(b"ab"),
            Bytes::copy_from_slice(b"cde"),
        ]);
        assert_eq!(Bytes::copy_from_slice(b"abc"), d.get_bytes(3));
        assert_eq!(2, d.remaining());
    }
}
