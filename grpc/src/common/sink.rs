use bytes::Bytes;
use client::types::ClientTypes;
use common::http_sink::HttpSink;
use common::types::Types;
use error;
use futures::Poll;
use httpbis;
use marshall::Marshaller;
use proto::grpc_frame::write_grpc_frame_to_vec;
use result;
use server::types::ServerTypes;
use arc_or_static::ArcOrStatic;


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
    fn poll(&mut self) -> Poll<(), httpbis::StreamDead>;
    fn send_data(&mut self, message: Bytes) -> result::Result<()>;
}

pub(crate) struct SinkCommonUntyped<T: Types> {
    pub(crate) http: T::HttpSink,
}

impl<T: Types> SinkCommonUntyped<T> {
    pub fn send_data(&mut self, message: Bytes) -> result::Result<()> {
        // TODO: allocation
        self.http
            .send_data(Bytes::from(write_grpc_frame_to_vec(&message)))?;
        Ok(())
    }
}

pub(crate) struct SinkCommon<M: 'static, T: Types> {
    pub marshaller: ArcOrStatic<Marshaller<M>>,
    pub sink: T::SinkUntyped,
}

impl<M: 'static, T: Types> SinkCommon<M, T> {
    pub fn poll(&mut self) -> Poll<(), httpbis::StreamDead> {
        self.sink.poll()
    }

    pub fn send_data(&mut self, message: M) -> result::Result<()> {
        let bytes = self.marshaller.write(&message)?;
        // TODO: extra allocation
        self.sink.send_data(Bytes::from(bytes))?;
        Ok(())
    }
}

fn _assert_types() {
    ::assert_types::assert_send::<SinkCommon<String, ClientTypes>>();
    ::assert_types::assert_send::<SinkCommon<String, ServerTypes>>();
}
