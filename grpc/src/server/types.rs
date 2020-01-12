use crate::common::types::Types;
use crate::server::resp_sink_untyped::ServerResponseUntypedSink;

pub(crate) struct ServerTypes;

impl Types for ServerTypes {
    type HttpSink = httpbis::ServerResponse;
    type SinkUntyped = ServerResponseUntypedSink;
}

unsafe impl Sync for ServerTypes {}
