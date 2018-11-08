use client::http_response_to_grpc_frames::http_response_to_grpc_frames;
use marshall::Marshaller;
use std::sync::Arc;
use StreamingResponse;

pub(crate) fn http_response_to_grpc_frames_typed<Resp: Send>(
    resp: httpbis::Response,
    marshaller: Arc<Marshaller<Resp>>,
) -> StreamingResponse<Resp> {
    http_response_to_grpc_frames(resp).and_then_items(move |message| marshaller.read(message))
}
