///! Convert HTTP response stream to gRPC stream
use std::collections::VecDeque;

use futures::future::TryFutureExt;
use futures::stream::Stream;

use httpbis::Headers;

use bytes::Bytes;

use crate::result;

use crate::error::Error;
use crate::error::GrpcMessageError;

use crate::proto::grpc_frame_parser::GrpcFrameParser;
use crate::proto::grpc_status::GrpcStatus;
use crate::proto::headers::HEADER_GRPC_MESSAGE;
use crate::proto::headers::HEADER_GRPC_STATUS;
use crate::proto::metadata::Metadata;
use crate::resp::*;
use crate::stream_item::*;
use futures::task::Context;
use httpbis::DataOrTrailers;
use httpbis::HttpStreamAfterHeaders;
use std::pin::Pin;
use std::task::Poll;

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
    StreamingResponse::new(response.0.map_err(|e| crate::Error::from(e)).and_then(
        |(headers, rem)| async {
            let metadata = init_headers_to_metadata(headers)?;
            let frames: GrpcStreamWithTrailingMetadata<Bytes> = GrpcStreamWithTrailingMetadata::new(
                GrpcFrameFromHttpFramesStreamResponse::new(rem),
            );
            Ok((metadata, frames))
        },
    ))
}

struct GrpcFrameFromHttpFramesStreamResponse {
    http_stream_stream: HttpStreamAfterHeaders,
    buf: GrpcFrameParser,
    parsed_frames: VecDeque<Bytes>,
}

impl GrpcFrameFromHttpFramesStreamResponse {
    pub fn new(http_stream_stream: HttpStreamAfterHeaders) -> Self {
        GrpcFrameFromHttpFramesStreamResponse {
            http_stream_stream,
            buf: GrpcFrameParser::default(),
            parsed_frames: VecDeque::new(),
        }
    }
}

impl Stream for GrpcFrameFromHttpFramesStreamResponse {
    type Item = crate::Result<ItemOrMetadata<Bytes>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let frames = self.buf.next_frames()?.0;
            self.parsed_frames.extend(frames);

            if let Some(frame) = self.parsed_frames.pop_front() {
                return Poll::Ready(Some(Ok(ItemOrMetadata::Item(frame))));
            }

            let part_opt =
                match unsafe { Pin::new_unchecked(&mut self.http_stream_stream) }.poll_next(cx)? {
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
                DataOrTrailers::Trailers(headers) => {
                    self.buf.check_empty()?;
                    let grpc_status = headers.get_opt_parse(HEADER_GRPC_STATUS);
                    if grpc_status == Some(GrpcStatus::Ok as i32) {
                        return Poll::Ready(Some(Ok(ItemOrMetadata::TrailingMetadata(
                            Metadata::from_headers(headers)?,
                        ))));
                    } else {
                        return Poll::Ready(Some(Err(
                            if let Some(message) = headers.get_opt(HEADER_GRPC_MESSAGE) {
                                Error::GrpcMessage(GrpcMessageError {
                                    grpc_status: grpc_status.unwrap_or(GrpcStatus::Unknown as i32),
                                    grpc_message: message.to_owned(),
                                })
                            } else {
                                Error::Other("not xxx")
                            },
                        )));
                    }
                }
                DataOrTrailers::Data(data, ..) => {
                    self.buf.enqueue(data);
                }
            }
        }
    }
}
