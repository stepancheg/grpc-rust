pub trait Marshaller<M> {
    fn write(&self, m: &M) -> Vec<u8>;
    fn read(&self, bytes: &[u8]) -> M;
}

struct MethodDescriptor<Req, Resp> {
    input_streaming: bool,
    output_streaming: bool,
    name: String,
    req_marshaller: Box<Marshaller<Req>>,
    resp_marshaller: Box<Marshaller<Resp>>,
}

trait MethodHandler<Req, Resp> {
    fn handle(&self, req: Req) -> Resp;
}

trait MethodHandlerDispatch {
    fn on_message(&self, message: &[u8]) -> Vec<u8>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    methodHandler: Box<MethodHandler<Req, Resp>>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp> {
    fn on_message(&self, message: &[u8]) -> Vec<u8> {
        panic!();
    }
}

struct ServerMethod {
    name: String,
    dispatch: Box<MethodHandlerDispatch>,
}

struct ServerServiceDefinition {
    methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    fn handle_method(&self, name: &str, message: &[u8]) -> Vec<u8> {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .unwrap()
            .dispatch
            .on_message(message)
    }
}
