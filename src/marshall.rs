use error::*;
use result;


pub trait Marshaller<M> {
    fn write(&self, m: &M) -> result::Result<Vec<u8>>;
    fn read(&self, bytes: &[u8]) -> result::Result<M>;
}

pub struct MarshallerString;

impl Marshaller<String> for MarshallerString {
    fn write(&self, m: &String) -> result::Result<Vec<u8>> {
        Ok(m.as_bytes().to_vec())
    }
    
    fn read(&self, bytes: &[u8]) -> result::Result<String> {
        String::from_utf8(bytes.to_vec())
            .map_err(|_| Error::Other("failed to parse utf-8"))
    }
}

pub struct MarshallerBytes;

impl Marshaller<Vec<u8>> for MarshallerBytes {
    fn write(&self, m: &Vec<u8>) -> result::Result<Vec<u8>> {
        Ok(m.clone())
    }
    
    fn read(&self, bytes: &[u8]) -> result::Result<Vec<u8>> {
        Ok(bytes.to_owned())
    }
}
