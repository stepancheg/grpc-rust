use bytes::Bytes;

pub trait Marshaller<M>: Send + Sync + 'static {
    fn write(&self, m: &M) -> crate::Result<Vec<u8>>;
    fn read(&self, bytes: Bytes) -> crate::Result<M>;
}
