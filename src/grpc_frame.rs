use futures::Async;
use futures::Poll;
use futures::stream;
use futures::stream::Stream;

use error::*;
use result::*;

use httpbis::http_common::*;



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


pub const GRPC_HEADER_LEN: usize = 5;


/// Return frame len
pub fn parse_grpc_frame_0(stream: &[u8]) -> GrpcResult<Option<usize>> {
    if stream.len() < GRPC_HEADER_LEN {
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
    let end = len + GRPC_HEADER_LEN;
    if end > stream.len() {
        return Ok(None);
    }

    Ok(Some(len))
}


// return message and size consumed
pub fn parse_grpc_frame(stream: &[u8]) -> GrpcResult<Option<(&[u8], usize)>> {
    parse_grpc_frame_0(stream)
        .map(|o| {
            o.map(|len| {
                (&stream[GRPC_HEADER_LEN .. len + GRPC_HEADER_LEN], len + GRPC_HEADER_LEN)
            })
        })
}

pub fn parse_grpc_frames_completely(stream: &[u8]) -> GrpcResult<Vec<&[u8]>> {
    let mut r = Vec::new();
    let mut pos = 0;
    while pos < stream.len() {
        let frame_opt = parse_grpc_frame(&stream[pos..])?;
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

#[allow(dead_code)]
pub fn parse_grpc_frame_completely(stream: &[u8]) -> GrpcResult<&[u8]> {
    let frames = parse_grpc_frames_completely(stream)?;
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

pub fn write_grpc_frame_to_vec(frame: &[u8]) -> Vec<u8> {
    let mut r = Vec::new();
    write_grpc_frame(&mut r, frame);
    r
}



trait RequestOrResponse {
    fn need_trailing_header() -> bool;
}

pub struct GrpcFrameFromHttpFramesStreamRequest {
    http_stream_stream: HttpPartStream,
    buf: Vec<u8>,
    error: Option<stream::Once<Vec<u8>, GrpcError>>,
}

impl GrpcFrameFromHttpFramesStreamRequest {
    pub fn new(http_stream_stream: HttpPartStream) -> Self {
        GrpcFrameFromHttpFramesStreamRequest {
            http_stream_stream: http_stream_stream,
            buf: Vec::new(),
            error: None,
        }
    }
}


impl Stream for GrpcFrameFromHttpFramesStreamRequest {
    type Item = Vec<u8>;
    type Error = GrpcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(ref mut error) = self.error {
                return error.poll();
            }

            if let Some((frame, len)) = parse_grpc_frame(&self.buf)?.map(|(frame, len)| (frame.to_owned(), len)) {
                self.buf.drain(..len);
                return Ok(Async::Ready(Some(frame)));
            }

            let part_opt = try_ready!(self.http_stream_stream.poll());
            let part = match part_opt {
                None => {
                    if self.buf.is_empty() {
                        return Ok(Async::Ready(None));
                    } else {
                        self.error = Some(stream::once(Err(GrpcError::Other("partial frame"))));
                        continue;
                    }
                },
                Some(part) => part,
            };

            match part.content {
                // unexpected but OK
                HttpStreamPartContent::Headers(..) => (),
                HttpStreamPartContent::Data(data) => self.buf.extend(data),
            }
        }
    }
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
