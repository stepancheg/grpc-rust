use std::ptr;
use std::mem;

use error::*;
use result::*;


// return message and size consumed
pub fn parse_grpc_frame(stream: &[u8]) -> GrpcResult<Option<(&[u8], usize)>> {
    let header_len = 5;
    if stream.len() < header_len {
        return Ok(None);
    }
    let compressed = match stream[0] {
        0 => false,
        1 => true,
        _ => return Err(GrpcError::Other("unknown compression flag")),
    };
    if compressed {
        return Err(GrpcError::Other("compression is not implemented"));
    }
    let mut len_raw = 0u32;
    unsafe {
        ptr::copy_nonoverlapping(&stream[1] as *const u8, &mut len_raw as *mut u32 as *mut u8, 4);
    }
    let len = len_raw.to_be() as usize;
    let end = len + header_len;
    if end > stream.len() {
        return Ok(None);
    }

    Ok(Some((&stream[header_len..end], end)))
}

pub fn parse_grpc_frame_completely(stream: &[u8]) -> GrpcResult<&[u8]> {
    match parse_grpc_frame(stream) {
        Ok(Some((bytes, pos))) if pos == stream.len() => Ok(bytes),
        Ok(..) => Err(GrpcError::Other("not complete single frame")),
        Err(e) => Err(e),
    }
}

pub fn write_grpc_frame(stream: &mut Vec<u8>, frame: &[u8]) {
	stream.push(0); // compressed flag
	let len_raw: [u8; 4] = unsafe {
	    mem::transmute((frame.len() as u32).to_be())
	};
	stream.extend(&len_raw);
	stream.extend(frame);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_frame() {
        assert_eq!(None, parse_grpc_frame(b"").unwrap());
        assert_eq!(None, parse_grpc_frame(b"1").unwrap());
        assert_eq!(None, parse_grpc_frame(b"14sc").unwrap());
        assert_eq!(
            None,
            parse_grpc_frame(b"\x00\x00\x00\x00\x07\x0a\x05wo").unwrap());
        assert_eq!(
            Some((&b"\x0a\x05world"[..], 12)),
            parse_grpc_frame(b"\x00\x00\x00\x00\x07\x0a\x05world").unwrap());
    }
}
