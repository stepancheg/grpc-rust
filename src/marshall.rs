use bytes::Bytes;

use result::Result;


pub trait Marshaller<M> {
    fn write(&self, m: &M) -> Result<Vec<u8>>;
    fn read(&self, bytes: Bytes) -> Result<M>;
}
