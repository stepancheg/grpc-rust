use crate::client::types::ClientTypes;
use crate::common::http_sink::HttpSink;
use crate::common::types::Types;
use crate::error;
use bytes::Bytes;

use crate::marshall::Marshaller;
use crate::or_static::arc::ArcOrStatic;
use crate::proto::grpc_frame::write_grpc_frame_cb;
use crate::result;
use crate::server::types::ServerTypes;
use futures::task::Context;
use httpbis;
use std::task::Poll;

pub trait MessageToBeSerialized {
    fn size_estimate(&self) -> result::Result<u32>;
    fn write(&self, size_estimate: u32, out: &mut Vec<u8>) -> result::Result<()>;
}

struct MessageToBeSerializedImpl<'a, T> {
    pub message: T,
    pub marshaller: &'a dyn Marshaller<T>,
}

impl<'a, T: 'static> MessageToBeSerialized for MessageToBeSerializedImpl<'a, T> {
    fn size_estimate(&self) -> result::Result<u32> {
        self.marshaller.write_size_estimate(&self.message)
    }

    fn write(&self, size_estimate: u32, out: &mut Vec<u8>) -> result::Result<()> {
        self.marshaller.write(&self.message, size_estimate, out)
    }
}

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
    fn send_message(&mut self, message: &dyn MessageToBeSerialized) -> result::Result<()>;
}

pub(crate) struct SinkCommonUntyped<T: Types> {
    pub(crate) http: T::HttpSink,
}

impl<T: Types> SinkCommonUntyped<T> {
    pub fn send_message(&mut self, message: &dyn MessageToBeSerialized) -> result::Result<()> {
        let mut data = Vec::new();
        let size_estimate = message.size_estimate()?;
        write_grpc_frame_cb(&mut data, size_estimate, |data| {
            message.write(size_estimate, data)
        })?;
        self.http.send_data(Bytes::from(data))?;
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
        self.sink.send_message(&MessageToBeSerializedImpl {
            message,
            marshaller: &*self.marshaller,
        })?;
        Ok(())
    }
}

fn _assert_types() {
    crate::assert_types::assert_send::<SinkCommon<String, ClientTypes>>();
    crate::assert_types::assert_send::<SinkCommon<String, ServerTypes>>();
}
