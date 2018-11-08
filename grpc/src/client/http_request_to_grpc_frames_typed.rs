use client::req_sink::ClientRequestSinkUntyped;
use common::sink::SinkCommon;
use common::sink::SinkCommonUntyped;
use marshall::Marshaller;
use std::sync::Arc;
use ClientRequestSink;

pub(crate) fn http_req_to_grpc_frames_typed<Req: Send>(
    http_req: httpbis::ClientRequest,
    req_marshaller: Arc<Marshaller<Req>>,
) -> ClientRequestSink<Req> {
    ClientRequestSink {
        common: SinkCommon {
            marshaller: req_marshaller,
            sink: ClientRequestSinkUntyped {
                common: SinkCommonUntyped { http: http_req },
            },
        },
    }
}
