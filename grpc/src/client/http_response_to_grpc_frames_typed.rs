use client::http_response_to_grpc_frames::http_response_to_grpc_frames;
use marshall::Marshaller;
use StreamingResponse;
use or_static::arc::ArcOrStatic;

pub(crate) fn http_response_to_grpc_frames_typed<Resp: Send>(
    resp: httpbis::Response,
    marshaller: ArcOrStatic<Marshaller<Resp>>,
) -> StreamingResponse<Resp> {
    http_response_to_grpc_frames(resp).and_then_items(move |message| marshaller.read(message))
}
