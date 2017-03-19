use std::mem;
use std::ops::Deref;

use bytes::*;


pub enum BytesOrVec {
    Bytes(Bytes),
    Vec(Vec<u8>),
}

impl BytesOrVec {
    pub fn new() -> BytesOrVec {
        BytesOrVec::Vec(Vec::new())
    }

    pub fn into_bytes(self) -> Bytes {
        match self {
            BytesOrVec::Bytes(b) => b,
            BytesOrVec::Vec(v) => Bytes::from(v),
        }
    }

    pub fn split_to(&mut self, to: usize) -> Bytes {
        let mut bytes = mem::replace(self, BytesOrVec::new()).into_bytes();
        let r = bytes.split_to(to);
        *self = BytesOrVec::Bytes(bytes);
        r
    }

    pub fn append(&mut self, data: Bytes) {
        if data.is_empty() {
            return;
        }

        *self = match mem::replace(self, BytesOrVec::new()) {
            BytesOrVec::Vec(mut v) => {
                if v.is_empty() {
                    BytesOrVec::Bytes(data)
                } else {
                    v.extend(&data);
                    BytesOrVec::Vec(v)
                }
            },
            BytesOrVec::Bytes(b) => {
                if b.is_empty() {
                    BytesOrVec::Bytes(data)
                } else {
                    let mut v = Vec::with_capacity(b.len() + data.len());
                    v.extend(&b);
                    v.extend(&data);
                    BytesOrVec::Vec(v)
                }
            }
        };
    }
}

impl Deref for BytesOrVec {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            &BytesOrVec::Bytes(ref b) => &b,
            &BytesOrVec::Vec(ref v) => &v,
        }
    }
}

