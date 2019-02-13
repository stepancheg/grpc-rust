use bytes::Bytes;

use result;

pub trait Marshaller<M>: Send + Sync + 'static {
    fn write(&self, m: &M) -> result::Result<Vec<u8>>;
    fn read(&self, bytes: Bytes) -> result::Result<M>;
}
