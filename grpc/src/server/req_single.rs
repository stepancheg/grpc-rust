use crate::Metadata;
use std::mem;

pub struct ServerRequestSingle<Req> {
    pub metadata: Metadata,
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
