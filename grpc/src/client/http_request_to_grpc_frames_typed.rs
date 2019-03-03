use client::req_sink::ClientRequestSinkUntyped;
use common::sink::SinkCommon;
use common::sink::SinkCommonUntyped;
use marshall::Marshaller;
use ClientRequestSink;
use or_static::arc::ArcOrStatic;

pub(crate) fn http_req_to_grpc_frames_typed<Req: Send + 'static>(
    http_req: httpbis::ClientRequest,
    req_marshaller: ArcOrStatic<Marshaller<Req>>,
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
