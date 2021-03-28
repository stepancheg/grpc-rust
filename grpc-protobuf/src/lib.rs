#![deny(broken_intra_doc_links)]

//! Implementation of marshaller for rust-protobuf parameter types.

use bytes::Bytes;

use grpc::marshall::Marshaller;

use protobuf::CodedInputStream;
use protobuf::Message;

pub struct MarshallerProtobuf;

impl<M: Message> Marshaller<M> for MarshallerProtobuf {
    fn write_size_estimate(&self, _m: &M) -> grpc::Result<u32> {
        // TODO: implement it
        Ok(0)
    }

    fn write(&self, m: &M, _size_esimate: u32, out: &mut Vec<u8>) -> grpc::Result<()> {
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
