use client::req_sink::ClientRequestSinkUntyped;
use common::types::Types;

pub(crate) struct ClientTypes;

impl Types for ClientTypes {
    type HttpSink = httpbis::ClientRequest;
    type SinkUntyped = ClientRequestSinkUntyped;
}

unsafe impl Sync for ClientTypes {}
