use result;
use server::resp_sink::ServerResponseSink;
use GrpcStatus;
use Metadata;

pub struct ServerResponseUnarySink<Resp: Send + 'static> {
    pub(crate) sink: ServerResponseSink<Resp>,
}

impl<Resp: Send + 'static> ServerResponseUnarySink<Resp> {
    pub fn send_metadata(&mut self, metadata: Metadata) -> result::Result<()> {
        self.sink.send_metadata(metadata)
    }

    pub fn finish_with_trailers(mut self, resp: Resp, metadata: Metadata) -> result::Result<()> {
        self.sink.send_data(resp)?;
        self.sink.send_trailers(metadata)?;
        Ok(())
    }

    pub fn finish(self, resp: Resp) -> result::Result<()> {
        self.finish_with_trailers(resp, Metadata::new())
    }

    pub fn send_grpc_error(mut self, status: GrpcStatus, message: String) -> result::Result<()> {
        self.sink.send_grpc_error(status, message)
    }
}
