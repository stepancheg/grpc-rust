//! Implementation of marshaller for protobuf parameter types.

extern crate grpc;
extern crate bytes;
extern crate protobuf;

use bytes::Bytes;

use grpc::marshall::Marshaller;

use protobuf::Message;
use protobuf::CodedInputStream;

pub struct MarshallerProtobuf;

impl<M: Message> Marshaller<M> for MarshallerProtobuf {
    fn write(&self, m: &M) -> grpc::Result<Vec<u8>> {
        m.write_to_bytes().map_err(|e| grpc::Error::Marshaller(Box::new(e)))
    }

    fn read(&self, buf: Bytes) -> grpc::Result<M> {
        // TODO: make protobuf simple
        let mut is = CodedInputStream::from_carllerche_bytes(&buf);
        let mut r: M = M::new();
        r.merge_from(&mut is).map_err(|e| grpc::Error::Marshaller(Box::new(e)))?;
        r.check_initialized().map_err(|e| grpc::Error::Marshaller(Box::new(e)))?;
        Ok(r)
    }
}
