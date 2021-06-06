use crate::client::http_response_to_grpc_frames::http_response_to_grpc_frames;
use crate::marshall::Marshaller;
use crate::or_static::arc::ArcOrStatic;
use crate::StreamingResponse;
use httpbis::Headers;
use httpbis::StreamAfterHeadersBox;
use httpbis::TryFutureBox;

pub(crate) fn http_response_to_grpc_frames_typed<Resp: Send>(
    resp: TryFutureBox<(Headers, StreamAfterHeadersBox)>,
    marshaller: ArcOrStatic<dyn Marshaller<Resp>>,
) -> StreamingResponse<Resp> {
    http_response_to_grpc_frames(resp).and_then_items(move |message| marshaller.read(message))
}
