use futures::Async;
use futures::Poll;
use futures::stream;
use futures::stream::Stream;

use error::*;
use result::*;
use grpc::*;

use httpbis::http_common::*;
use httpbis::solicit_misc::*;



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
    http_stream_stream: HttpPartFutureStreamSend,
    buf: Vec<u8>,
    error: Option<stream::Once<Vec<u8>, GrpcError>>,
}

pub struct GrpcFrameFromHttpFramesStreamResponse {
    http_stream_stream: HttpPartFutureStreamSend,
    buf: Vec<u8>,
    seen_headers: bool,
    error: Option<stream::Once<Vec<u8>, GrpcError>>,
}

impl GrpcFrameFromHttpFramesStreamResponse {
    pub fn new(http_stream_stream: HttpPartFutureStreamSend) -> Self {
        GrpcFrameFromHttpFramesStreamResponse {
            http_stream_stream: http_stream_stream,
            buf: Vec::new(),
            seen_headers: false,
            error: None,
        }
    }
}

impl GrpcFrameFromHttpFramesStreamRequest {
    pub fn new(http_stream_stream: HttpPartFutureStreamSend) -> Self {
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

impl Stream for GrpcFrameFromHttpFramesStreamResponse {
    type Item = Vec<u8>;
    type Error = GrpcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(ref mut error) = self.error {
                return error.poll();
            }

            if !self.seen_headers {
                let headers = try_ready!(self.http_stream_stream.poll());
                let headers = match headers {
                    Some(headers) => headers,
                    None => return Ok(Async::Ready(None)),
                };
                match headers.content {
                    HttpStreamPartContent::Headers(headers) => {
                        if slice_get_header(&headers, ":status") != Some("200") {
                            self.error = Some(stream::once(Err(GrpcError::Other("not 200"))));
                            continue;
                        }

                        // Check gRPC status code and message
                        // TODO: a more detailed error message.
                        if slice_get_header(&headers, HEADER_GRPC_STATUS) != Some("0") {
                            let message = slice_get_header(&headers, HEADER_GRPC_MESSAGE).unwrap_or("unknown error");
                            self.error = Some(stream::once(Err(
                                GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })
                            )));
                            continue;
                        }
                    }
                    HttpStreamPartContent::Data(..) => {
                        self.error = Some(stream::once(Err(GrpcError::Other("data before headers"))));
                        continue;
                    }
                };
                self.seen_headers = true;
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
                HttpStreamPartContent::Headers(headers) => {
                    if part.last {
                        if !self.buf.is_empty() {
                            self.error = Some(stream::once(Err(GrpcError::Other("partial frame"))));
                        } else {
                            let grpc_status_0 = slice_get_header(&headers, HEADER_GRPC_STATUS) == Some("0");
                            if grpc_status_0 {
                                return Ok(Async::Ready(None));
                            } else {
                                self.error = Some(stream::once(Err(if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                                    GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })
                                } else {
                                    GrpcError::Other("not xxx")
                                })));
                            }
                        }
                        continue;
                    }
                },
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
