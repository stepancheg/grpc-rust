use std::collections::VecDeque;

use bytes::Buf;
use bytes::Bytes;

use futures::stream::Stream;
use futures::stream::StreamExt;

use crate::error::*;
use crate::proto::grpc_frame_parser::GrpcFrameParser;
use crate::result;
use futures::task::Context;
use httpbis::DataOrTrailers;
use httpbis::HttpStreamAfterHeaders;
use std::pin::Pin;
use std::task::Poll;

pub const GRPC_HEADER_LEN: usize = 5;

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

/// Encode data into grpc frame (add frame prefix)
pub fn write_grpc_frame_cb<F, E>(stream: &mut Vec<u8>, estimate: u32, frame: F) -> Result<(), E>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), E>,
{
    stream.reserve(estimate as usize + GRPC_HEADER_LEN);

    stream.push(0); // compressed flag
    let size_pos = stream.len();
    stream.extend_from_slice(&[0, 0, 0, 0]); // len
    let frame_start = stream.len();
    frame(stream)?;
    let frame_size = stream.len() - frame_start;
    assert!(frame_size <= u32::max_value() as usize);
    stream[size_pos..size_pos + 4].copy_from_slice(&(frame_size as u32).to_be_bytes());
    Ok(())
}

trait RequestOrResponse {
    fn need_trailing_header() -> bool;
}

pub struct GrpcFrameFromHttpFramesStreamRequest {
    http_stream_stream: HttpStreamAfterHeaders,
    buf: GrpcFrameParser,
    parsed_frames: VecDeque<Bytes>,
}

impl GrpcFrameFromHttpFramesStreamRequest {
    pub fn _new(http_stream_stream: HttpStreamAfterHeaders) -> Self {
        GrpcFrameFromHttpFramesStreamRequest {
            http_stream_stream,
            buf: GrpcFrameParser::default(),
            parsed_frames: VecDeque::new(),
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
            let parsed_frames = self.buf.next_frames()?.0;

            self.parsed_frames.extend(parsed_frames);

            if let Some(frame) = self.parsed_frames.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }

            let part_opt = match self.http_stream_stream.poll_next_unpin(cx)? {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(part_opt) => part_opt,
            };
            let part = match part_opt {
                None => {
                    self.buf.check_empty()?;
                    return Poll::Ready(None);
                }
                Some(part) => part,
            };

            match part {
                // unexpected but OK
                DataOrTrailers::Trailers(..) => (),
                DataOrTrailers::Data(data, ..) => {
                    self.buf.enqueue(data);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::BytesMut;

    /// Return frame len
    fn parse_grpc_frame_0(stream: &[u8]) -> result::Result<Option<usize>> {
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

    fn parse_grpc_frame_from_bytes(stream: &mut Bytes) -> result::Result<Option<Bytes>> {
        if let Some(len) = parse_grpc_frame_0(&stream)? {
            let r = stream.slice(GRPC_HEADER_LEN..len + GRPC_HEADER_LEN);
            stream.advance(len + GRPC_HEADER_LEN);
            Ok(Some(r))
        } else {
            Ok(None)
        }
    }

    fn parse_grpc_frames_from_bytes(stream: &mut Bytes) -> result::Result<Vec<Bytes>> {
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

    // return message and size consumed
    fn parse_grpc_frame(stream: &[u8]) -> result::Result<Option<(&[u8], usize)>> {
        parse_grpc_frame_0(stream).map(|o| {
            o.map(|len| {
                (
                    &stream[GRPC_HEADER_LEN..len + GRPC_HEADER_LEN],
                    len + GRPC_HEADER_LEN,
                )
            })
        })
    }

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
            let mut b = BytesMut::new();
            b.extend_from_slice(input);
            b.extend_from_slice(trail);
            let mut b = b.freeze();

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
        write_grpc_frame_cb(&mut f, 0, |f| {
            Ok::<_, ()>(f.extend_from_slice(&[0x17, 0x19]))
        })
        .unwrap();
        assert_eq!(&b"\x00\x00\x00\x00\x02\x17\x19"[..], f.as_slice());
    }
}
