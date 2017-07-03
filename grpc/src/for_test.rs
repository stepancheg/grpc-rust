//! Code useful in tests.


use bytes::Bytes;

use result::Result;
use error::Error;
use marshall::Marshaller;



pub struct MarshallerString;

impl Marshaller<String> for MarshallerString {
    fn write(&self, m: &String) -> Result<Vec<u8>> {
        Ok(m.as_bytes().to_vec())
    }

    fn read(&self, bytes: Bytes) -> Result<String> {
        String::from_utf8(bytes.as_ref().to_vec())
            .map_err(|_| Error::Other("failed to parse utf-8"))
    }
}

pub struct MarshallerBytes;

impl Marshaller<Vec<u8>> for MarshallerBytes {
    fn write(&self, m: &Vec<u8>) -> Result<Vec<u8>> {
        Ok(m.clone())
    }

    fn read(&self, bytes: Bytes) -> Result<Vec<u8>> {
        Ok(bytes.as_ref().to_vec())
    }
}
