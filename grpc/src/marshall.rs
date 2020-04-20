use bytes::Bytes;

/// Message (request or response) marshaller
pub trait Marshaller<M>: Send + Sync + 'static {
    /// Estimate message size.
    ///
    /// Can be used for more efficient capacity allocation.
    ///
    /// Default implementation returns zero.
    fn write_size_estimate(&self, _m: &M) -> crate::Result<u32> {
        Ok(0)
    }

    /// Serialize a message into a `Vec<u8>`.
    fn write(&self, m: &M, estimated_size: u32, out: &mut Vec<u8>) -> crate::Result<()>;

    /// Parse a message from given `Bytes` object.
    fn read(&self, bytes: Bytes) -> crate::Result<M>;
}
