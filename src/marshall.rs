use error::*;
use result::*;

// TODO: GrpcResult
pub trait Marshaller<M> {
    fn write(&self, m: &M) -> GrpcResult<Vec<u8>>;
    fn read(&self, bytes: &[u8]) -> GrpcResult<M>;
}

pub struct MarshallerString;

impl Marshaller<String> for MarshallerString {
    fn write(&self, m: &String) -> GrpcResult<Vec<u8>> {
        Ok(m.as_bytes().to_vec())
    }
    
    fn read(&self, bytes: &[u8]) -> GrpcResult<String> {
        String::from_utf8(bytes.to_vec())
            .map_err(|_| GrpcError::Other("failed to parse utf-8"))
    }
}

pub struct MarshallerBytes;

impl Marshaller<Vec<u8>> for MarshallerBytes {
    fn write(&self, m: &Vec<u8>) -> GrpcResult<Vec<u8>> {
        Ok(m.clone())
    }
    
    fn read(&self, bytes: &[u8]) -> GrpcResult<Vec<u8>> {
        Ok(bytes.to_owned())
    }
}
