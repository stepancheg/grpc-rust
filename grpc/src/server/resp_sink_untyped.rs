use crate::common::sink::SinkUntyped;
use crate::common::sink::{MessageToBeSerialized, SinkCommonUntyped};

use crate::proto::grpc_status::GrpcStatus;
use crate::proto::headers::headers_200;
use crate::proto::headers::headers_500;
use crate::proto::headers::trailers;
use crate::result;
use crate::server::types::ServerTypes;
use crate::Metadata;
use futures::task::Context;
use httpbis::SenderState;
use std::task::Poll;

pub(crate) struct ServerResponseUntypedSink {
    pub common: SinkCommonUntyped<ServerTypes>,
}

impl SinkUntyped for ServerResponseUntypedSink {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>> {
        self.common.http.poll(cx)
    }

    fn send_message(&mut self, message: &dyn MessageToBeSerialized) -> result::Result<()> {
        if self.common.http.state() == httpbis::SenderState::ExpectingHeaders {
            self.send_metadata(Metadata::new())?;
        }
        self.common.send_message(message)
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
