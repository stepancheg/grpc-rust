//! Code useful in tests.

use bytes::Bytes;

use crate::error::Error;
use crate::marshall::Marshaller;
use crate::result::Result;

pub struct MarshallerString;

impl Marshaller<String> for MarshallerString {
    fn write(&self, m: &String, _estimated_size: u32, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(m.as_bytes());
        Ok(())
    }

    fn read(&self, bytes: Bytes) -> Result<String> {
        String::from_utf8(bytes.as_ref().to_vec())
            .map_err(|_| Error::Other("failed to parse utf-8"))
    }
}

pub struct MarshallerBytes;

impl Marshaller<Vec<u8>> for MarshallerBytes {
    fn write(&self, m: &Vec<u8>, _estimated_size: u32, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&m);
        Ok(())
    }

    fn read(&self, bytes: Bytes) -> Result<Vec<u8>> {
        Ok(bytes.as_ref().to_vec())
    }
}
