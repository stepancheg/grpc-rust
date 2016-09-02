use marshall::*;


pub enum GrpcStreaming {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Bidi,
}


pub struct MethodDescriptor<Req, Resp> {
    pub name: String,
    pub streaming: GrpcStreaming,
    pub req_marshaller: Box<Marshaller<Req> + Sync + Send>,
    pub resp_marshaller: Box<Marshaller<Resp> + Sync + Send>,
}

