use marshall::*;

struct MethodDescriptor<Req, Resp> {
    name: String,
    input_streaming: bool,
    output_streaming: bool,
    req_marshaller: Box<Marshaller<Req>>,
    resp_marshaller: Box<Marshaller<Resp>>,
}

trait MethodHandler<Req, Resp> {
    fn handle(&self, req: Req) -> Resp;
}

struct MethodHandlerEcho;

impl<A> MethodHandler<A, A> for MethodHandlerEcho {
    fn handle(&self, req: A) -> A {
        req
    }
} 

trait MethodHandlerDispatch {
    fn on_message(&self, message: &[u8]) -> Vec<u8>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: MethodDescriptor<Req, Resp>,
    method_handler: Box<MethodHandler<Req, Resp>>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp> {
    fn on_message(&self, message: &[u8]) -> Vec<u8> {
        let req = self.desc.req_marshaller.read(message);
        let resp = self.method_handler.handle(req);
        self.desc.resp_marshaller.write(&resp)
    }
}

pub struct ServerMethod {
    name: String,
    dispatch: Box<MethodHandlerDispatch>,
}

impl ServerMethod {
    fn new<Req : 'static, Resp : 'static>(method: MethodDescriptor<Req, Resp>, handler: Box<MethodHandler<Req, Resp>>) -> ServerMethod {
        ServerMethod {
            name: method.name.clone(),
            dispatch: Box::new(MethodHandlerDispatchImpl {
                desc: method,
                method_handler: handler,    
            }),
        }
    }
}

pub struct ServerServiceDefinition {
    methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    pub fn new(mut methods: Vec<ServerMethod>) -> ServerServiceDefinition {
        methods.push(
            ServerMethod::new(
                MethodDescriptor {
                    name: "/helloworld.Greeter/SayHello".to_owned(),
                    input_streaming: false,
                    output_streaming: false,
                    req_marshaller: Box::new(MarshallerBytes),
                    resp_marshaller: Box::new(MarshallerBytes),
                },
                Box::new(MethodHandlerEcho)
            )
        );
        ServerServiceDefinition {
            methods: methods,
        }
    }

    pub fn handle_method(&self, name: &str, message: &[u8]) -> Vec<u8> {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
            .dispatch
            .on_message(message)
    }
}
