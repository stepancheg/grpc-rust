use std::marker;

use method::Marshaller;

use protobuf::Message;
use protobuf::MessageStatic;
use protobuf::CodedInputStream;

struct ProtobufMarshaller<M> {
    _marker: marker::PhantomData<M>,
}

impl<M : Message + MessageStatic> Marshaller<M> for ProtobufMarshaller<M> {
    fn write(&self, m: &M) -> Vec<u8> {
        m.write_to_bytes().unwrap()
    }

    fn read(&self, buf: &[u8]) -> M {
        // TODO: make protobuf simple
        let mut is = CodedInputStream::from_bytes(buf);
        let mut r: M = M::new();
        r.merge_from(&mut is).unwrap();
        r.check_initialized();
        r
    }
}
