use crate::Metadata;
use std::mem;

/// Unary request.
///
/// This object is passed to unary request handlers.
pub struct ServerRequestSingle<Req> {
    /// Request metadata.
    pub metadata: Metadata,
    /// The message.
    pub message: Req,
}

impl<Req> ServerRequestSingle<Req> {
    // Return contained message and replace it with `Default::default`
    pub fn take_message(&mut self) -> Req
    where
        Req: Default,
    {
        mem::replace(&mut self.message, Default::default())
    }
}
