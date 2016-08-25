// TODO: GrpcResult
pub trait Marshaller<M> {
    fn write(&self, m: &M) -> Vec<u8>;
    fn read(&self, bytes: &[u8]) -> M;
}

pub struct MarshallerString;

impl Marshaller<String> for MarshallerString {
    fn write(&self, m: &String) -> Vec<u8> {
    	m.as_bytes().to_vec()
    }
    
    fn read(&self, bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

pub struct MarshallerBytes;

impl Marshaller<Vec<u8>> for MarshallerBytes {
    fn write(&self, m: &Vec<u8>) -> Vec<u8> {
        m.clone()
    }
    
    fn read(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_owned()
    }
}
