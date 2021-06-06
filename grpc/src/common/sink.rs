use crate::client::types::ClientTypes;
use crate::common::types::Types;
use bytes::Bytes;

use crate::marshall::Marshaller;
use crate::or_static::arc::ArcOrStatic;
use crate::proto::grpc_frame::write_grpc_frame_cb;
use crate::result;
use crate::server::types::ServerTypes;
use futures::task::Context;
use httpbis;
use httpbis::SinkAfterHeadersBox;
use std::marker;
use std::task::Poll;

pub trait MessageToBeSerialized {
    fn size_estimate(&self) -> result::Result<u32>;
    fn write(&self, size_estimate: u32, out: &mut Vec<u8>) -> result::Result<()>;

    fn serialize_to_grpc_frame(&self) -> result::Result<Bytes> {
        let mut data = Vec::new();
        let size_estimate = self.size_estimate()?;
        write_grpc_frame_cb(&mut data, size_estimate, |data| {
            self.write(size_estimate, data)
        })?;
        Ok(Bytes::from(data))
    }
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

/// Client request or server response.
pub(crate) trait SinkUntyped {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::Error>>;
    fn send_message(&mut self, message: &dyn MessageToBeSerialized) -> result::Result<()>;
}

pub(crate) struct SinkCommonUntyped<T: Types> {
    pub(crate) http: SinkAfterHeadersBox,
    pub(crate) _marker: marker::PhantomData<T>,
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
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::Error>> {
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
