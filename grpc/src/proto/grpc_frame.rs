use std::collections::VecDeque;

use bytes::{Buf, Bytes};

use futures::future;
use futures::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;

use crate::bytesx::bytes_extend;
use crate::error::*;
use crate::result;
use futures::task::Context;
use httpbis::DataOrTrailers;
use httpbis::HttpStreamAfterHeaders;
use std::pin::Pin;
use std::task::Poll;

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
    let mut stream_copy = stream;
    let len = match parse_grpc_frame_header(&mut stream_copy)? {
        Some(len) => len,
        None => return Ok(None),
    };

    let end = len as usize + GRPC_HEADER_LEN;
    if end > stream.len() {
        return Ok(None);
    }

    Ok(Some(len as usize))
}

pub fn parse_grpc_frame_header<B: Buf>(stream: &mut B) -> result::Result<Option<u32>> {
    if stream.remaining() < GRPC_HEADER_LEN {
        return Ok(None);
    }

    let compressed = match stream.get_u8() {
        0 => false,
        1 => true,
        _ => return Err(Error::Other("unknown compression flag")),
    };
    if compressed {
        return Err(Error::Other("compression is not implemented"));
    }
    Ok(Some(stream.get_u32()))
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
        let r = stream.slice(GRPC_HEADER_LEN..len + GRPC_HEADER_LEN);
        stream.advance(len + GRPC_HEADER_LEN);
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
pub fn write_grpc_frame_cb<F, E>(stream: &mut Vec<u8>, frame: F) -> Result<(), E>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), E>,
{
    stream.push(0); // compressed flag
    let size_pos = stream.len();
    stream.extend_from_slice(&[0, 0, 0, 0]); // len
    let frame_start = stream.len();
    frame(stream)?;
    let frame_size = stream.len() - frame_start;
    assert!(frame_size <= u32::max_value() as usize);
    stream[size_pos..size_pos + 4].copy_from_slice(&write_u32_be(frame_size as u32));
    Ok(())
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
    error: Option<stream::Once<future::Ready<crate::Result<Bytes>>>>,
}

impl GrpcFrameFromHttpFramesStreamRequest {
    pub fn _new(http_stream_stream: HttpStreamAfterHeaders) -> Self {
        GrpcFrameFromHttpFramesStreamRequest {
            http_stream_stream,
            buf: Bytes::new(),
            parsed_frames: VecDeque::new(),
            error: None,
        }
    }
}

impl Stream for GrpcFrameFromHttpFramesStreamRequest {
    type Item = crate::Result<Bytes>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<crate::Result<Bytes>>> {
        loop {
            if let Some(ref mut error) = self.error {
                return error.poll_next_unpin(cx);
            }

            let p_r = parse_grpc_frames_from_bytes(&mut self.buf);

            self.parsed_frames.extend(match p_r {
                Ok(r) => r,
                Err(e) => {
                    self.error = Some(stream::once(future::ready(Err(e))));
                    continue;
                }
            });

            if let Some(frame) = self.parsed_frames.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }

            let part_opt = match self.http_stream_stream.poll_next_unpin(cx)? {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(part_opt) => part_opt,
            };
            let part = match part_opt {
                None => {
                    if self.buf.is_empty() {
                        return Poll::Ready(None);
                    } else {
                        self.error = Some(stream::once(future::err(Error::Other("partial frame"))));
                        continue;
                    }
                }
                Some(part) => part,
            };

            match part {
                // unexpected but OK
                DataOrTrailers::Trailers(..) => (),
                DataOrTrailers::Data(data, ..) => {
                    bytes_extend(&mut self.buf, data);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bytesx::bytes_extend_from_slice;

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
            bytes_extend_from_slice(&mut b, input);
            bytes_extend_from_slice(&mut b, trail);

            let r: Vec<Bytes> = r.into_iter().map(|&s| Bytes::copy_from_slice(s)).collect();

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

    #[test]
    fn test_write_grpc_frame_cb() {
        let mut f = Vec::new();
        write_grpc_frame_cb(&mut f, |f| Ok::<_, ()>(f.extend_from_slice(&[0x17, 0x19]))).unwrap();
        assert_eq!(&b"\x00\x00\x00\x00\x02\x17\x19"[..], f.as_slice());
    }
}
