use crate::result;
use crate::server::resp_sink::ServerResponseSink;
use crate::GrpcStatus;
use crate::Metadata;

/// A sink for single message (for unary request).
pub struct ServerResponseUnarySink<Resp: Send + 'static> {
    pub(crate) sink: ServerResponseSink<Resp>,
}

impl<Resp: Send + 'static> ServerResponseUnarySink<Resp> {
    /// Send response header metadata.
    ///
    /// This operation can be only called before data sent.
    pub fn send_metadata(&mut self, metadata: Metadata) -> result::Result<()> {
        self.sink.send_metadata(metadata)
    }

    /// Send the response with trailers metadata.
    pub fn finish_with_trailers(mut self, resp: Resp, metadata: Metadata) -> result::Result<()> {
        self.sink.send_data(resp)?;
        self.sink.send_trailers(metadata)?;
        Ok(())
    }

    /// Send the response.
    pub fn finish(self, resp: Resp) -> result::Result<()> {
        self.finish_with_trailers(resp, Metadata::new())
    }

    /// Send error.
    pub fn send_grpc_error(mut self, status: GrpcStatus, message: String) -> result::Result<()> {
        self.sink.send_grpc_error(status, message)
    }
}
