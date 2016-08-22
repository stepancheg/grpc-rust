use marshall::*;

use result::GrpcResult;


pub struct MethodDescriptor<Req, Resp> {
    pub name: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub req_marshaller: Box<Marshaller<Req> + Sync + Send>,
    pub resp_marshaller: Box<Marshaller<Resp> + Sync + Send>,
}

pub trait MethodHandler<Req, Resp> {
    fn handle(&self, req: Req) -> GrpcResult<Resp>;
}

pub struct MethodHandlerEcho;

impl<A> MethodHandler<A, A> for MethodHandlerEcho {
    fn handle(&self, req: A) -> GrpcResult<A> {
        println!("handle echo");
        Ok(req)
    }
}

pub struct MethodHandlerFn<F> {
    f: F
}

impl<F> MethodHandlerFn<F> {
    pub fn new<Req, Resp>(f: F)
        -> Self
        where F : Fn(Req) -> GrpcResult<Resp>
    {
        MethodHandlerFn {
            f: f,
        }
    }
}

impl<Req, Resp, F : Fn(Req) -> GrpcResult<Resp>> MethodHandler<Req, Resp> for MethodHandlerFn<F> {
    fn handle(&self, req: Req) -> GrpcResult<Resp> {
        (self.f)(req)
    }
}

trait MethodHandlerDispatch {
    fn on_message(&self, message: &[u8]) -> GrpcResult<Vec<u8>>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: MethodDescriptor<Req, Resp>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp> {
    fn on_message(&self, message: &[u8]) -> GrpcResult<Vec<u8>> {
        let req = self.desc.req_marshaller.read(message);
        let resp = try!(self.method_handler.handle(req));
        Ok(self.desc.resp_marshaller.write(&resp))
    }
}

pub struct ServerMethod {
    name: String,
    dispatch: Box<MethodHandlerDispatch + Sync + Send>,
}

impl ServerMethod {
    pub fn new<Req : 'static, Resp : 'static, H : MethodHandler<Req, Resp> + 'static + Sync + Send>(method: MethodDescriptor<Req, Resp>, handler: H) -> ServerMethod {
        ServerMethod {
            name: method.name.clone(),
            dispatch: Box::new(MethodHandlerDispatchImpl {
                desc: method,
                method_handler: Box::new(handler),
            }),
        }
    }
}

pub struct ServerServiceDefinition {
    methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    pub fn new(mut methods: Vec<ServerMethod>) -> ServerServiceDefinition {
        ServerServiceDefinition {
            methods: methods,
        }
    }

    pub fn find_method(&self, name: &str) -> &ServerMethod {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
    }

    pub fn handle_method(&self, name: &str, message: &[u8]) -> GrpcResult<Vec<u8>> {
        self.find_method(name).dispatch.on_message(message)
    }
}
