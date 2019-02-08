///! Convert HTTP response stream to gRPC stream
use std::collections::VecDeque;

use futures::future::Future;
use futures::stream;
use futures::stream::Stream;
use futures::Async;
use futures::Poll;

use httpbis;
use httpbis::Headers;

use bytes::Bytes;

use result;

use error::Error;
use error::GrpcMessageError;

use httpbis::DataOrTrailers;
use httpbis::HttpStreamAfterHeaders;
use metadata::*;
use resp::*;
use stream_item::*;
use proto::grpc_status::HEADER_GRPC_STATUS;
use proto::grpc_status::GrpcStatus;
use proto::grpc_status::HEADER_GRPC_MESSAGE;
use proto::grpc_frame::parse_grpc_frames_from_bytes;

fn init_headers_to_metadata(headers: Headers) -> result::Result<Metadata> {
    if headers.get_opt(":status") != Some("200") {
        return Err(Error::Other("not 200"));
    }

    // Check gRPC status code and message
    // TODO: a more detailed error message.
    if let Some(grpc_status) = headers.get_opt_parse(HEADER_GRPC_STATUS) {
        if grpc_status != GrpcStatus::Ok as i32 {
            let message = headers
                .get_opt(HEADER_GRPC_MESSAGE)
                .unwrap_or("unknown error");
            return Err(Error::GrpcMessage(GrpcMessageError {
                grpc_status: grpc_status,
                grpc_message: message.to_owned(),
            }));
        }
    }

    Ok(Metadata::from_headers(headers)?)
}

pub fn http_response_to_grpc_frames(response: httpbis::Response) -> StreamingResponse<Bytes> {
    StreamingResponse::new(
        response
            .0
            .map_err(|e| Error::from(e))
            .and_then(|(headers, rem)| {
                let metadata = init_headers_to_metadata(headers)?;
                let frames: GrpcStreamWithTrailingMetadata<Bytes> =
                    GrpcStreamWithTrailingMetadata::new(
                        GrpcFrameFromHttpFramesStreamResponse::new(rem),
                    );
                Ok((metadata, frames))
            }),
    )
}

struct GrpcFrameFromHttpFramesStreamResponse {
    http_stream_stream: HttpStreamAfterHeaders,
    buf: Bytes,
    parsed_frames: VecDeque<Bytes>,
    error: Option<stream::Once<ItemOrMetadata<Bytes>, Error>>,
}

impl GrpcFrameFromHttpFramesStreamResponse {
    pub fn new(http_stream_stream: HttpStreamAfterHeaders) -> Self {
        GrpcFrameFromHttpFramesStreamResponse {
            http_stream_stream,
            buf: Bytes::new(),
            parsed_frames: VecDeque::new(),
            error: None,
        }
    }
}

impl Stream for GrpcFrameFromHttpFramesStreamResponse {
    type Item = ItemOrMetadata<Bytes>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
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
                return Ok(Async::Ready(Some(ItemOrMetadata::Item(frame))));
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
                DataOrTrailers::Trailers(headers) => {
                    if !self.buf.is_empty() {
                        self.error = Some(stream::once(Err(Error::Other("partial frame"))));
                    } else {
                        let grpc_status = headers.get_opt_parse(HEADER_GRPC_STATUS);
                        if grpc_status == Some(GrpcStatus::Ok as i32) {
                            return Ok(Async::Ready(Some(ItemOrMetadata::TrailingMetadata(
                                Metadata::from_headers(headers)?,
                            ))));
                        } else {
                            self.error = Some(stream::once(Err(if let Some(message) =
                                headers.get_opt(HEADER_GRPC_MESSAGE)
                            {
                                Error::GrpcMessage(GrpcMessageError {
                                    grpc_status: grpc_status.unwrap_or(GrpcStatus::Unknown as i32),
                                    grpc_message: message.to_owned(),
                                })
                            } else {
                                Error::Other("not xxx")
                            })));
                        }
                    }
                    continue;
                }
                DataOrTrailers::Data(data, ..) => {
                    self.buf.extend_from_slice(&data);
                }
            }
        }
    }
}
