use crate::common::sink::SinkCommon;

use crate::proto::grpc_status::GrpcStatus;
use crate::proto::metadata::Metadata;
use crate::result;
use crate::server::types::ServerTypes;
use httpbis;

use futures::task::Context;
use std::task::Poll;

pub struct ServerResponseSink<Resp: Send + 'static> {
    pub(crate) common: SinkCommon<Resp, ServerTypes>,
}

impl<Resp: Send> ServerResponseSink<Resp> {
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>> {
        self.common.poll(cx)
    }

    pub fn send_metadata(&mut self, metadata: Metadata) -> result::Result<()> {
        self.common.sink.send_metadata(metadata)?;
        Ok(())
    }

    pub fn send_data(&mut self, message: Resp) -> result::Result<()> {
        self.common.send_data(message)
    }

    pub fn send_trailers(&mut self, metadata: Metadata) -> result::Result<()> {
        self.common.sink.send_trailers(metadata)?;
        Ok(())
    }

    pub fn send_grpc_error(&mut self, status: GrpcStatus, message: String) -> result::Result<()> {
        self.common.sink.send_grpc_error(status, message)?;
        Ok(())
    }
}
