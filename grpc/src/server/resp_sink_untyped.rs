use std::task::Poll;

use crate::common::sink::MessageToBeSerialized;
use crate::common::sink::SinkUntyped;

use crate::proto::grpc_status::GrpcStatus;
use crate::proto::headers::headers_200;
use crate::proto::headers::headers_500;
use crate::proto::headers::trailers;
use crate::result;
use crate::Metadata;
use futures::task::Context;
use httpbis::ServerResponse;
use httpbis::SinkAfterHeaders;
use httpbis::SinkAfterHeadersBox;
use std::mem;

pub(crate) enum ServerResponseUntypedSink {
    Headers(ServerResponse),
    AfterHeaders(SinkAfterHeadersBox),
    Done,
}

impl SinkUntyped for ServerResponseUntypedSink {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::Error>> {
        match self {
            ServerResponseUntypedSink::Headers(_) => Poll::Ready(Ok(())),
            ServerResponseUntypedSink::AfterHeaders(x) => x.poll(cx),
            ServerResponseUntypedSink::Done => Poll::Ready(Ok(())),
        }
    }

    fn send_message(&mut self, message: &dyn MessageToBeSerialized) -> result::Result<()> {
        if let ServerResponseUntypedSink::Headers(..) = self {
            self.send_metadata(Metadata::new())?;
        }
        match self {
            ServerResponseUntypedSink::Headers(_) => unreachable!(),
            ServerResponseUntypedSink::AfterHeaders(r) => {
                Ok(r.send_data(message.serialize_to_grpc_frame()?)?)
            }
            ServerResponseUntypedSink::Done => Err(crate::Error::Other("done")),
        }
    }
}

impl ServerResponseUntypedSink {
    pub fn send_metadata(&mut self, metadata: Metadata) -> Result<(), crate::Error> {
        match mem::replace(self, ServerResponseUntypedSink::Done) {
            ServerResponseUntypedSink::Headers(resp) => {
                *self = ServerResponseUntypedSink::AfterHeaders(
                    resp.send_headers(headers_200(metadata))?,
                );
                Ok(())
            }
            s @ ServerResponseUntypedSink::AfterHeaders(_)
            | s @ ServerResponseUntypedSink::Done => {
                *self = s;
                Err(crate::Error::Other("after headers"))
            }
        }
    }

    pub fn send_trailers(&mut self, metadata: Metadata) -> Result<(), crate::Error> {
        match mem::replace(self, ServerResponseUntypedSink::Done) {
            ServerResponseUntypedSink::Headers(_) => Err(crate::Error::Other("headers")),
            ServerResponseUntypedSink::AfterHeaders(mut x) => {
                Ok(x.send_trailers(trailers(GrpcStatus::Ok, None, metadata))?)
            }
            ServerResponseUntypedSink::Done => Err(crate::Error::Other("done")),
        }
    }

    pub fn send_grpc_error(
        &mut self,
        grpc_status: GrpcStatus,
        message: String,
    ) -> Result<(), crate::Error> {
        // TODO: if headers sent, then ...
        let headers = headers_500(grpc_status, message.to_owned());

        match mem::replace(self, ServerResponseUntypedSink::Done) {
            ServerResponseUntypedSink::Headers(resp) => {
                resp.send_headers(headers)?;
                Ok(())
            }
            ServerResponseUntypedSink::AfterHeaders(mut resp) => {
                resp.send_trailers(headers)?;
                Ok(())
            }
            ServerResponseUntypedSink::Done => Err(crate::Error::Other("Already closed")),
        }
    }
}
