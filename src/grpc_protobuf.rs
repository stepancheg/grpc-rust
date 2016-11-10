use marshall::Marshaller;

use protobuf::Message;
use protobuf::MessageStatic;
use protobuf::CodedInputStream;

use result::*;
use error::*;


pub struct MarshallerProtobuf;

impl<M : Message + MessageStatic> Marshaller<M> for MarshallerProtobuf {
    fn write(&self, m: &M) -> GrpcResult<Vec<u8>> {
        m.write_to_bytes()
            .map_err(GrpcError::from)
    }

    fn read(&self, buf: &[u8]) -> GrpcResult<M> {
        // TODO: make protobuf simple
        let mut is = CodedInputStream::from_bytes(buf);
        let mut r: M = M::new();
        r.merge_from(&mut is)?;
        r.check_initialized()?;
        Ok(r)
    }
}
