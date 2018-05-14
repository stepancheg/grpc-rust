//! Implementation of marshaller for protobuf parameter types.

use bytes::Bytes;

use marshall::Marshaller;

use protobuf_lib::Message;
use protobuf_lib::CodedInputStream;

use result;


pub struct MarshallerProtobuf;

impl<M : Message> Marshaller<M> for MarshallerProtobuf {
    fn write(&self, m: &M) -> result::Result<Vec<u8>> {
        Ok(m.write_to_bytes()?)
    }

    fn read(&self, buf: Bytes) -> result::Result<M> {
        // TODO: make protobuf simple
        let mut is = CodedInputStream::from_carllerche_bytes(&buf);
        let mut r: M = M::new();
        r.merge_from(&mut is)?;
        r.check_initialized()?;
        Ok(r)
    }
}
