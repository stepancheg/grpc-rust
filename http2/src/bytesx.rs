use std::mem;

use bytes::*;

// inefficient: https://github.com/carllerche/bytes/issues/85
#[allow(dead_code)]
pub fn bytes_concat(mut a: Bytes, b: Bytes) -> Bytes {
    if b.is_empty() {
        a
    } else if a.is_empty() {
        b
    } else {
        bytes_extend_with(&mut a, b);
        a
    }
}

pub fn bytes_extend_with_slice(target: &mut Bytes, append: &[u8]) {
    let mut bytes_mut = BytesMut::from(mem::replace(target, Bytes::new()));
    bytes_mut_extend_with_slice(&mut bytes_mut, append);
    mem::replace(target, bytes_mut.freeze());
}

pub fn bytes_extend_with(target: &mut Bytes, append: Bytes) {
    // TODO: optimize
    bytes_extend_with_slice(target, &append[..]);
}

// https://github.com/carllerche/bytes/pull/112
pub fn bytes_mut_extend_with_slice(target: &mut BytesMut, append: &[u8]) {
    target.reserve(append.len());
    target.put_slice(append);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bytes_concat() {
        let a = Bytes::from("aabb");
        let b = Bytes::from("ccddee");
        let r = bytes_concat(a, b);
        let e = Bytes::from("aabbccddee");
        assert_eq!(e, r);
    }
}
