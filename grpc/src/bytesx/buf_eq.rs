use bytes::Buf;
use std::cmp;

/// Test if two buffers content is the same.
pub fn buf_eq<B1: Buf, B2: Buf>(mut b1: B1, mut b2: B2) -> bool {
    loop {
        if !b1.has_remaining() || !b2.has_remaining() {
            return b1.has_remaining() == b2.has_remaining();
        }
        let s1 = b1.chunk();
        let s2 = b2.chunk();
        let min = cmp::min(s1.len(), s2.len());
        if &s1[..min] != &s2[..min] {
            return false;
        }
        b1.advance(min);
        b2.advance(min);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert!(buf_eq(b"".as_ref(), b"".as_ref()));
        assert!(buf_eq(b"ab".as_ref(), b"ab".as_ref()));
        assert!(!buf_eq(b"a".as_ref(), b"ab".as_ref()));
        assert!(!buf_eq(b"abc".as_ref(), b"ab".as_ref()));

        assert!(buf_eq(b"".as_ref().chain(b"ab".as_ref()), b"ab".as_ref()));
        assert!(buf_eq(b"a".as_ref().chain(b"b".as_ref()), b"ab".as_ref()));
        assert!(!buf_eq(b"a".as_ref().chain(b"bc".as_ref()), b"ab".as_ref()));
    }
}
