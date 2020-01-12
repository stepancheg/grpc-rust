use common::sink::SinkCommon;
use futures::future;
use futures::future::Future;
use futures::Poll;
use httpbis;
use httpbis::StreamDead;
use proto::grpc_status::GrpcStatus;
use proto::metadata::Metadata;
use result;
use server::types::ServerTypes;

pub struct ServerResponseSink<Resp: Send + 'static> {
    pub(crate) common: SinkCommon<Resp, ServerTypes>,
}

impl<Resp: Send> ServerResponseSink<Resp> {
    pub fn poll(&mut self) -> Poll<(), httpbis::StreamDead> {
        self.common.poll()
    }

    pub fn block_wait(&mut self) -> Result<(), StreamDead> {
        future::poll_fn(|| self.poll()).wait()
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
