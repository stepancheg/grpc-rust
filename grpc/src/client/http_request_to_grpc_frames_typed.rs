use crate::client::req_sink::ClientRequestSinkUntyped;
use crate::common::sink::SinkCommon;
use crate::common::sink::SinkCommonUntyped;
use crate::marshall::Marshaller;
use crate::or_static::arc::ArcOrStatic;
use crate::ClientRequestSink;
use httpbis::SinkAfterHeadersBox;
use std::marker;

pub(crate) fn http_req_to_grpc_frames_typed<Req: Send + 'static>(
    http_req: SinkAfterHeadersBox,
    req_marshaller: ArcOrStatic<dyn Marshaller<Req>>,
) -> ClientRequestSink<Req> {
    ClientRequestSink {
        common: SinkCommon {
            marshaller: req_marshaller,
            sink: ClientRequestSinkUntyped {
                common: SinkCommonUntyped {
                    http: http_req,
                    _marker: marker::PhantomData,
                },
            },
        },
    }
}
