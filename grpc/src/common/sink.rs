use crate::client::types::ClientTypes;
use crate::common::http_sink::HttpSink;
use crate::common::types::Types;
use crate::error;
use bytes::Bytes;

use crate::marshall::Marshaller;
use crate::or_static::arc::ArcOrStatic;
use crate::proto::grpc_frame::write_grpc_frame_to_vec;
use crate::result;
use crate::server::types::ServerTypes;
use futures::task::Context;
use httpbis;
use std::task::Poll;

pub enum SendError {
    Http(httpbis::SendError),
    _Marshall(error::Error),
}

impl From<httpbis::SendError> for SendError {
    fn from(e: httpbis::SendError) -> Self {
        SendError::Http(e)
    }
}

pub(crate) trait SinkUntyped {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>>;
    fn send_message(&mut self, message: Bytes) -> result::Result<()>;
}

pub(crate) struct SinkCommonUntyped<T: Types> {
    pub(crate) http: T::HttpSink,
}

impl<T: Types> SinkCommonUntyped<T> {
    pub fn send_message(&mut self, message: Bytes) -> result::Result<()> {
        // TODO: allocation
        self.http
            .send_data(Bytes::from(write_grpc_frame_to_vec(&message)))?;
        Ok(())
    }
}

pub(crate) struct SinkCommon<M: 'static, T: Types> {
    pub marshaller: ArcOrStatic<dyn Marshaller<M>>,
    pub sink: T::SinkUntyped,
}

impl<M: 'static, T: Types> SinkCommon<M, T> {
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>> {
        self.sink.poll(cx)
    }

    pub fn send_data(&mut self, message: M) -> result::Result<()> {
        let mut bytes = Vec::new();
        let size_estimate = self.marshaller.write_size_estimate(&message)?;
        self.marshaller.write(&message, size_estimate, &mut bytes)?;
        // TODO: extra allocation
        self.sink.send_message(Bytes::from(bytes))?;
        Ok(())
    }
}

fn _assert_types() {
    crate::assert_types::assert_send::<SinkCommon<String, ClientTypes>>();
    crate::assert_types::assert_send::<SinkCommon<String, ServerTypes>>();
}
