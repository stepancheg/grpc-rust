use crate::Metadata;
use std::mem;
use tokio::runtime::Handle;

/// Unary request.
///
/// This object is passed to unary request handlers.
pub struct ServerRequestSingle<'a, Req> {
    pub(crate) loop_handle: &'a Handle,
    /// Request metadata.
    pub metadata: Metadata,
    /// The message.
    pub message: Req,
}

impl<'a, Req> ServerRequestSingle<'a, Req> {
    pub fn loop_handle(&self) -> Handle {
        self.loop_handle.clone()
    }

    // Return contained message and replace it with `Default::default`
    pub fn take_message(&mut self) -> Req
    where
        Req: Default,
    {
        mem::replace(&mut self.message, Default::default())
    }
}
