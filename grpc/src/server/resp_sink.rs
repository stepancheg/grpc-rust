use crate::common::sink::SinkCommon;

use crate::proto::grpc_status::GrpcStatus;
use crate::proto::metadata::Metadata;
use crate::result;
use crate::server::types::ServerTypes;
use httpbis;

use futures::future;
use futures::task::Context;
use std::task::Poll;

/// Sink for server gRPC response.
pub struct ServerResponseSink<Resp: Send + 'static> {
    pub(crate) common: SinkCommon<Resp, ServerTypes>,
}

impl<Resp: Send> ServerResponseSink<Resp> {
    /// This method returns [`Poll::Ready`] when the sink is available to accept data
    /// otherwise subscribe current task which will be notified when this
    /// stream is ready to accept data.
    ///
    /// Higher level wrapper (future) is [`ready`](ServerResponseSink::ready).
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>> {
        self.common.poll(cx)
    }

    /// A future is resolved when the sink is available to receive more data.
    pub async fn ready(&mut self) -> Result<(), httpbis::StreamDead> {
        future::poll_fn(|cx| self.poll(cx)).await
    }

    /// Send response metadata.
    ///
    /// This function returns error if metadata is already sent.
    pub fn send_metadata(&mut self, metadata: Metadata) -> result::Result<()> {
        self.common.sink.send_metadata(metadata)?;
        Ok(())
    }

    /// Send response data.
    ///
    /// This function does not block, but if send window is not available,
    /// data is queued.
    ///
    /// [`ready`](ServerResponseSink::ready) function can be used to wait for windows availability.
    pub fn send_data(&mut self, message: Resp) -> result::Result<()> {
        self.common.send_data(message)
    }

    /// Send trailers after the response.
    ///
    /// This function must be called after data is sent, otherwise the response stream is reset.
    pub fn send_trailers(&mut self, metadata: Metadata) -> result::Result<()> {
        self.common.sink.send_trailers(metadata)?;
        Ok(())
    }

    /// Send error.
    pub fn send_grpc_error(&mut self, status: GrpcStatus, message: String) -> result::Result<()> {
        self.common.sink.send_grpc_error(status, message)?;
        Ok(())
    }
}
