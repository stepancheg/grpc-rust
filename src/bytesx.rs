use bytes::*;

// inefficient: https://github.com/carllerche/bytes/issues/85
#[allow(dead_code)]
pub fn bytes_concat(a: Bytes, b: Bytes) -> Bytes {
    if b.is_empty() {
        a
    } else if a.is_empty() {
        b
    } else {
        let mut r = BytesMut::with_capacity(a.len() + b.len());
        r.put_slice(&a);
        r.put_slice(&b);
        r.freeze()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::*;

    #[test]
    fn test_bytes_concat() {
        let a = Bytes::from("aabb");
        let b = Bytes::from("ccddee");
        let r = bytes_concat(a, b);
        let e = Bytes::from("aabbccddee");
        assert_eq!(e, r);
    }
}
