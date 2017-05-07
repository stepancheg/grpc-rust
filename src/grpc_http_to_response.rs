///! Convert HTTP response stream to gRPC stream


use futures::Async;
use futures::Poll;
use futures::future::Future;
use futures::stream;
use futures::stream::Stream;

use grpc::GrpcStatus;
use grpc::HEADER_GRPC_STATUS;
use grpc::HEADER_GRPC_MESSAGE;

use httpbis::solicit::header::Headers;
use httpbis::bytesx::bytes_extend_with;
use httpbis::http_common::HttpStreamPartContent;
use httpbis::http_common::HttpPartFutureStreamSend;

use futures_grpc::*;

use grpc_frame::*;

use bytes::Bytes;

use result::*;

use error::GrpcError;
use error::GrpcMessageError;

use resp::*;
use metadata::*;



fn init_headers_to_metadata(headers: Headers) -> GrpcResult<GrpcMetadata> {
    if headers.get_opt(":status") != Some("200") {
        return Err(GrpcError::Other("not 200"));
    }

    // Check gRPC status code and message
    // TODO: a more detailed error message.
    if let Some(grpc_status) = headers.get_opt_parse(HEADER_GRPC_STATUS) {
        if grpc_status != GrpcStatus::Ok as i32 {
            let message = headers.get_opt(HEADER_GRPC_MESSAGE).unwrap_or("unknown error");
            return Err(
                GrpcError::GrpcMessage(GrpcMessageError{
                    grpc_status: grpc_status,
                    grpc_message: message.to_owned(),
                })
            );
        }
    }

    Ok(GrpcMetadata::from_headers(headers)?)
}


pub fn http_response_to_grpc_frames(response: HttpPartFutureStreamSend) -> GrpcStreamingResponse<Bytes> {
    GrpcStreamingResponse::new(response.into_future().map_err(|(e, _s)| GrpcError::from(e)).and_then(|(part, rem)| {
        match part {
            Some(part) => {
                match part.content {
                    HttpStreamPartContent::Headers(headers) => {
                        let metadata = init_headers_to_metadata(headers)?;
                        let frames: GrpcStreamSend<Bytes> = Box::new(GrpcFrameFromHttpFramesStreamResponse::new(rem));
                        Ok((metadata, frames))
                    },
                    HttpStreamPartContent::Data(..) => {
                        Err(GrpcError::Other("data before headers"))
                    }
                }
            }
            None => {
                Err(GrpcError::Other("empty response"))
            }
        }
    }))
}


struct GrpcFrameFromHttpFramesStreamResponse {
    http_stream_stream: HttpPartFutureStreamSend,
    buf: Bytes,
    error: Option<stream::Once<Bytes, GrpcError>>,
}

impl GrpcFrameFromHttpFramesStreamResponse {
    pub fn new(http_stream_stream: HttpPartFutureStreamSend) -> Self {
        GrpcFrameFromHttpFramesStreamResponse {
            http_stream_stream: http_stream_stream,
            buf: Bytes::new(),
            error: None,
        }
    }
}

impl Stream for GrpcFrameFromHttpFramesStreamResponse {
    type Item = Bytes;
    type Error = GrpcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(ref mut error) = self.error {
                return error.poll();
            }

            if let Some(frame_len) = parse_grpc_frame_0(&self.buf)? {
                let frame = self.buf
                    .split_to(frame_len + GRPC_HEADER_LEN)
                    .slice_from(GRPC_HEADER_LEN);
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
                            let grpc_status = headers.get_opt_parse(HEADER_GRPC_STATUS);
                            if grpc_status == Some(GrpcStatus::Ok as i32) {
                                return Ok(Async::Ready(None));
                            } else {
                                self.error = Some(stream::once(Err(if let Some(message) = headers.get_opt(HEADER_GRPC_MESSAGE) {
                                    GrpcError::GrpcMessage(GrpcMessageError {
                                        grpc_status: grpc_status.unwrap_or(GrpcStatus::Unknown as i32),
                                        grpc_message: message.to_owned(),
                                    })
                                } else {
                                    GrpcError::Other("not xxx")
                                })));
                            }
                        }
                        continue;
                    }
                },
                HttpStreamPartContent::Data(data) => {
                    bytes_extend_with(&mut self.buf, data);
                }
            }
        }
    }
}
