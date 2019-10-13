use marshall::*;
use or_static::string::StringOrStatic;
use or_static::arc::ArcOrStatic;

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

pub struct MethodDescriptor<Req: 'static, Resp: 'static> {
    pub name: StringOrStatic,
    pub streaming: GrpcStreaming,
    pub req_marshaller: ArcOrStatic<dyn Marshaller<Req>>,
    pub resp_marshaller: ArcOrStatic<dyn Marshaller<Resp>>,
}
