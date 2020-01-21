//! Implementation of marshaller for protobuf parameter types.

extern crate bytes;
extern crate grpc;
extern crate protobuf;

use bytes::Bytes;

use grpc::marshall::Marshaller;

use protobuf::CodedInputStream;
use protobuf::Message;

pub struct MarshallerProtobuf;

impl<M: Message> Marshaller<M> for MarshallerProtobuf {
    fn write(&self, m: &M, out: &mut Vec<u8>) -> grpc::Result<()> {
        m.write_to_vec(out)
            .map_err(|e| grpc::Error::Marshaller(Box::new(e)))
    }

    fn read(&self, buf: Bytes) -> grpc::Result<M> {
        // TODO: make protobuf simple
        let mut is = CodedInputStream::from_carllerche_bytes(&buf);
        let mut r: M = M::new();
        r.merge_from(&mut is)
            .map_err(|e| grpc::Error::Marshaller(Box::new(e)))?;
        r.check_initialized()
            .map_err(|e| grpc::Error::Marshaller(Box::new(e)))?;
        Ok(r)
    }
}
