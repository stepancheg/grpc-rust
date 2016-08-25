use marshall::*;


pub struct MethodDescriptor<Req, Resp> {
    pub name: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub req_marshaller: Box<Marshaller<Req> + Sync + Send>,
    pub resp_marshaller: Box<Marshaller<Resp> + Sync + Send>,
}

