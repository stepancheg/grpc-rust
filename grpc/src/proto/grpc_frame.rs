use std::collections::VecDeque;

use bytes::Bytes;

use futures::stream;
use futures::stream::Stream;
use futures::Async;
use futures::Poll;

use error::*;
use httpbis::DataOrTrailers;
use httpbis::HttpStreamAfterHeaders;
use result;

fn read_u32_be(bytes: &[u8]) -> u32 {
    0 | ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | ((bytes[3] as u32) << 0)
}

fn write_u32_be(v: u32) -> [u8; 4] {
    [
        (v >> 24) as u8,
        (v >> 16) as u8,
        (v >> 8) as u8,
        (v >> 0) as u8,
    ]
}

pub const GRPC_HEADER_LEN: usize = 5;

/// Return frame len
pub fn parse_grpc_frame_0(stream: &[u8]) -> result::Result<Option<usize>> {
    if stream.len() < GRPC_HEADER_LEN {
        return Ok(None);
    }
    let compressed = match stream[0] {
        0 => false,
        1 => true,
        _ => return Err(Error::Other("unknown compression flag")),
    };
    if compressed {
        return Err(Error::Other("compression is not implemented"));
    }
    let len = read_u32_be(&stream[1..]) as usize;
    let end = len + GRPC_HEADER_LEN;
    if end > stream.len() {
        return Ok(None);
    }

    Ok(Some(len))
}

// return message and size consumed
pub fn parse_grpc_frame(stream: &[u8]) -> result::Result<Option<(&[u8], usize)>> {
    parse_grpc_frame_0(stream).map(|o| {
        o.map(|len| {
            (
                &stream[GRPC_HEADER_LEN..len + GRPC_HEADER_LEN],
                len + GRPC_HEADER_LEN,
            )
        })
    })
}

pub fn parse_grpc_frame_from_bytes(stream: &mut Bytes) -> result::Result<Option<Bytes>> {
    if let Some(len) = parse_grpc_frame_0(&stream)? {
        let r = stream.slice(GRPC_HEADER_LEN, len + GRPC_HEADER_LEN);
        stream.split_to(len + GRPC_HEADER_LEN);
        Ok(Some(r))
    } else {
        Ok(None)
    }
}

pub fn parse_grpc_frames_from_bytes(stream: &mut Bytes) -> result::Result<Vec<Bytes>> {
    let mut r = Vec::new();
    loop {
        match parse_grpc_frame_from_bytes(stream)? {
            Some(bytes) => {
                r.push(bytes);
            }
            None => {
                return Ok(r);
            }
        }
    }
}

pub fn parse_grpc_frames_completely(stream: &[u8]) -> result::Result<Vec<&[u8]>> {
    let mut r = Vec::new();
    let mut pos = 0;
    while pos < stream.len() {
        let frame_opt = parse_grpc_frame(&stream[pos..])?;
        match frame_opt {
            None => return Err(Error::Other("not complete frames")),
            Some((frame, len)) => {
                r.push(frame);
                pos += len;
            }
        }
    }
    Ok(r)
}

#[allow(dead_code)]
pub fn parse_grpc_frame_completely(stream: &[u8]) -> result::Result<&[u8]> {
    let frames = parse_grpc_frames_completely(stream)?;
    if frames.len() == 1 {
        Ok(frames[0])
    } else {
        Err(Error::Other("expecting exactly one frame"))
    }
}

/// Encode data into grpc frame (add frame prefix)
pub fn write_grpc_frame(stream: &mut Vec<u8>, frame: &[u8]) {
    assert!(frame.len() <= u32::max_value() as usize);
    stream.reserve(5 + frame.len());
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
    http_stream_stream: HttpStreamAfterHeaders,
    buf: Bytes,
    parsed_frames: VecDeque<Bytes>,
    error: Option<stream::Once<Bytes, Error>>,
}

impl GrpcFrameFromHttpFramesStreamRequest {
    pub fn new(http_stream_stream: HttpStreamAfterHeaders) -> Self {
        GrpcFrameFromHttpFramesStreamRequest {
            http_stream_stream,
            buf: Bytes::new(),
            parsed_frames: VecDeque::new(),
            error: None,
        }
    }
}

impl Stream for GrpcFrameFromHttpFramesStreamRequest {
    type Item = Bytes;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Bytes>, Error> {
        loop {
            if let Some(ref mut error) = self.error {
                return error.poll();
            }

            self.parsed_frames
                .extend(match parse_grpc_frames_from_bytes(&mut self.buf) {
                    Ok(r) => r,
                    Err(e) => {
                        self.error = Some(stream::once(Err(e)));
                        continue;
                    }
                });

            if let Some(frame) = self.parsed_frames.pop_front() {
                return Ok(Async::Ready(Some(frame)));
            }

            let part_opt = match self.http_stream_stream.poll()? {
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(part_opt) => part_opt,
            };
            let part = match part_opt {
                None => {
                    if self.buf.is_empty() {
                        return Ok(Async::Ready(None));
                    } else {
                        self.error = Some(stream::once(Err(Error::Other("partial frame"))));
                        continue;
                    }
                }
                Some(part) => part,
            };

            match part {
                // unexpected but OK
                DataOrTrailers::Trailers(..) => (),
                DataOrTrailers::Data(data, ..) => {
                    self.buf.extend_from_slice(&data);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_grpc_frame() {
        assert_eq!(None, parse_grpc_frame(b"").unwrap());
        assert_eq!(None, parse_grpc_frame(b"1").unwrap());
        assert_eq!(None, parse_grpc_frame(b"14sc").unwrap());
        assert_eq!(
            None,
            parse_grpc_frame(b"\x00\x00\x00\x00\x07\x0a\x05wo").unwrap()
        );
        assert_eq!(
            Some((&b"\x0a\x05world"[..], 12)),
            parse_grpc_frame(b"\x00\x00\x00\x00\x07\x0a\x05world").unwrap()
        );
    }

    #[test]
    fn test_parse_grpc_frames_from_bytes() {
        fn t(r: &[&[u8]], input: &[u8], trail: &[u8]) {
            let mut b = Bytes::new();
            b.extend_from_slice(input);
            b.extend_from_slice(trail);

            let r: Vec<Bytes> = r.into_iter().map(|&s| Bytes::from(s)).collect();

            let rr = parse_grpc_frames_from_bytes(&mut b).unwrap();
            assert_eq!(r, rr);
            assert_eq!(trail, b.as_ref());
        }

        t(&[], &[], &[]);
        t(&[], &[], &b"1"[..]);
        t(&[], &[], &b"14sc"[..]);
        t(
            &[&b"\x0a\x05world"[..]],
            &b"\x00\x00\x00\x00\x07\x0a\x05world"[..],
            &[],
        );
        t(
            &[&b"ab"[..], &b"cde"[..]],
            &b"\0\x00\x00\x00\x02ab\0\x00\x00\x00\x03cde"[..],
            &b"\x00"[..],
        );
    }
}
