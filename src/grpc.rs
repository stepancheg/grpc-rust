use std::ptr;

// return message and size consumed
pub fn parse_frame(stream: &[u8]) -> Option<(&[u8], usize)> {
    let header_len = 5;
    if stream.len() < header_len {
        return None;
    }
    let compressed = match stream[0] {
        0 => false,
        1 => panic!("compression is not implemented"),
        _ => panic!("unknown compression flag"),
    };
    let mut len_raw = 0u32;
    unsafe {
        ptr::copy_nonoverlapping(&stream[1] as *const u8, &mut len_raw as *mut u32 as *mut u8, 4);
    }
    let len = len_raw.to_be() as usize;
    let end = len + header_len;
    if end > stream.len() {
        return None;
    }

    Some((&stream[header_len..end], end))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_frame() {
        assert_eq!(None, parse_frame(b""));
        assert_eq!(None, parse_frame(b"1"));
        assert_eq!(None, parse_frame(b"14sc"));
        assert_eq!(
            None,
            parse_frame(b"\x00\x00\x00\x00\x07\x0a\x05wo"));
        assert_eq!(
            Some((&b"\x0a\x05world"[..], 12)),
            parse_frame(b"\x00\x00\x00\x00\x07\x0a\x05world"));
    }
}
