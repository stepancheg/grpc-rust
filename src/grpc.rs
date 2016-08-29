use error::*;
use result::*;


pub static HEADER_GRPC_STATUS: &'static str = "grpc-status";
pub static HEADER_GRPC_MESSAGE: &'static str = "grpc-message";


fn read_u32_be(bytes: &[u8]) -> u32 {
    0
        | ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) <<  8)
        | ((bytes[3] as u32) <<  0)
}

fn write_u32_be(v: u32) -> [u8; 4] {
    [
        (v >> 24) as u8,
        (v >> 16) as u8,
        (v >>  8) as u8,
        (v >>  0) as u8,
    ]
}


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
    let len = read_u32_be(&stream[1..]) as usize;
    let end = len + header_len;
    if end > stream.len() {
        return Ok(None);
    }

    Ok(Some((&stream[header_len..end], end)))
}

pub fn parse_grpc_frames_completely(stream: &[u8]) -> GrpcResult<Vec<&[u8]>> {
    let mut r = Vec::new();
    let mut pos = 0;
    while pos < stream.len() {
        let frame_opt = try!(parse_grpc_frame(&stream[pos..]));
        match frame_opt {
            None => return Err(GrpcError::Other("not complete frames")),
            Some((frame, len)) => {
                r.push(frame);
                pos += len;
            }
        }
    }
    Ok(r)
}

pub fn parse_grpc_frame_completely(stream: &[u8]) -> GrpcResult<&[u8]> {
    let frames = try!(parse_grpc_frames_completely(stream));
    if frames.len() == 1 {
        Ok(frames[0])
    } else {
        Err(GrpcError::Other("expecting exactly one frame"))
    }
}

pub fn write_grpc_frame(stream: &mut Vec<u8>, frame: &[u8]) {
	stream.push(0); // compressed flag
	stream.extend(&write_u32_be(frame.len() as u32));
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
