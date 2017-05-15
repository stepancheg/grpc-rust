//! Implementation of marshaller for protobuf parameter types.

use marshall::Marshaller;

use protobuf_lib::Message;
use protobuf_lib::MessageStatic;
use protobuf_lib::CodedInputStream;

use result;


pub struct MarshallerProtobuf;

impl<M : Message + MessageStatic> Marshaller<M> for MarshallerProtobuf {
    fn write(&self, m: &M) -> result::Result<Vec<u8>> {
        Ok(m.write_to_bytes()?)
    }

    fn read(&self, buf: &[u8]) -> result::Result<M> {
        // TODO: make protobuf simple
        let mut is = CodedInputStream::from_bytes(buf);
        let mut r: M = M::new();
        r.merge_from(&mut is)?;
        r.check_initialized()?;
        Ok(r)
    }
}
