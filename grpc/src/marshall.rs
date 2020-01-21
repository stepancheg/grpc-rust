use bytes::Bytes;

pub trait Marshaller<M>: Send + Sync + 'static {
    fn write(&self, m: &M, out: &mut Vec<u8>) -> crate::Result<()>;
    fn read(&self, bytes: Bytes) -> crate::Result<M>;
}
