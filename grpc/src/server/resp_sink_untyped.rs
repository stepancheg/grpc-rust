use bytes::Bytes;
use common::sink::SinkCommonUntyped;
use common::sink::SinkUntyped;
use futures::Poll;
use httpbis::SenderState;
use proto::grpc_status::GrpcStatus;
use proto::headers::headers_200;
use proto::headers::headers_500;
use proto::headers::trailers;
use result;
use server::types::ServerTypes;
use Metadata;

pub(crate) struct ServerResponseUntypedSink {
    pub common: SinkCommonUntyped<ServerTypes>,
}

impl SinkUntyped for ServerResponseUntypedSink {
    fn poll(&mut self) -> Poll<(), httpbis::StreamDead> {
        self.common.http.poll()
    }

    fn send_data(&mut self, message: Bytes) -> result::Result<()> {
        if self.common.http.state() == httpbis::SenderState::ExpectingHeaders {
            self.send_metadata(Metadata::new())?;
        }
        self.common.send_data(message)
    }
}

impl ServerResponseUntypedSink {
    pub fn send_metadata(&mut self, metadata: Metadata) -> Result<(), httpbis::SendError> {
        if self.common.http.state() != httpbis::SenderState::ExpectingHeaders {
            return Err(httpbis::SendError::IncorrectState(self.common.http.state()));
        }
        self.do_send_headers(metadata)
    }

    fn do_send_headers(&mut self, metadata: Metadata) -> Result<(), httpbis::SendError> {
        let headers = headers_200(metadata);
        self.common.http.send_headers(headers)
    }

    pub fn send_trailers(&mut self, metadata: Metadata) -> Result<(), httpbis::SendError> {
        self.common
            .http
            .send_trailers(trailers(GrpcStatus::Ok, None, metadata))
    }

    pub fn _close(&mut self) -> Result<(), httpbis::SendError> {
        self.send_trailers(Metadata::new())
    }

    pub fn send_grpc_error(
        &mut self,
        grpc_status: GrpcStatus,
        message: String,
    ) -> Result<(), httpbis::SendError> {
        // TODO: if headers sent, then ...
        let headers = headers_500(grpc_status, message.to_owned());

        if self.common.http.state() == SenderState::ExpectingHeaders {
            self.common.http.send_headers_end_of_stream(headers)
        } else {
            self.common.http.send_trailers(headers)
        }
    }
}
