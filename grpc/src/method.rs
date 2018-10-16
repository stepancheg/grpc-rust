use marshall::*;

pub enum GrpcStreaming {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Bidi,
}

pub trait GrpcStreamingFlavor {
    type Flavor;

    fn streaming() -> GrpcStreaming;
}

pub struct GrpcStreamingUnary;
pub struct GrpcStreamingClientStreaming;
pub struct GrpcStreamingServerStreaming;
pub struct GrpcStreamingBidi;

pub struct MethodDescriptor<Req, Resp> {
    pub name: String,
    pub streaming: GrpcStreaming,
    pub req_marshaller: Box<Marshaller<Req> + Sync + Send>,
    pub resp_marshaller: Box<Marshaller<Resp> + Sync + Send>,
}
