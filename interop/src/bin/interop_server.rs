extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate futures_cpupool;
extern crate grpc;
extern crate protobuf;

extern crate grpc_interop;
use grpc_interop::*;

use std::thread;

use bytes::Bytes;

use futures::stream;
use futures::stream::Stream;

use grpc::*;
use futures::Async;

static DICTIONARY: &'static str = "ABCDEFGHIJKLMNOPQRSTUVabcdefghijklmnoqprstuvwxyz0123456789";
// Note: due to const restrictions, this is calculated by hand.
static DICTIONARY_SIZE: usize = 58;

/**
 * Returns a Vec<u8> with a given size containing printable u8s.
 */
fn make_string(size: usize) -> Vec<u8> {
    let dict = DICTIONARY.to_owned().into_bytes();
    let mut result = Vec::<u8>::with_capacity(size);

    for n in 0..size {
        result.push(dict[n % DICTIONARY_SIZE]);
    }

    return result;
}

fn echo_custom_metadata(req_metadata: &Metadata) -> Metadata {
    static TEST_ECHO_KEY: &'static str = "x-grpc-test-echo-initial";
    let mut metadata = Metadata::new();
    if let Some(value) = req_metadata.get(TEST_ECHO_KEY) {
        metadata.add(MetadataKey::from(TEST_ECHO_KEY), Bytes::from(value));
    }
    metadata
}

fn echo_custom_trailing(req_metadata: &Metadata) -> Metadata {
    static TEST_ECHO_TRAILING_KEY: &'static str = "x-grpc-test-echo-trailing-bin";
    let mut metadata = Metadata::new();
    if let Some(value) = req_metadata.get(TEST_ECHO_TRAILING_KEY) {
        metadata.add(
            MetadataKey::from(TEST_ECHO_TRAILING_KEY),
            Bytes::from(value),
        );
    }
    metadata
}

struct TestServerImpl {}

impl TestService for TestServerImpl {
    fn empty_call(&self, _ctx: ServerHandlerContext, _req: ServerRequestSingle<Empty>, resp: ServerResponseUnarySink<Empty>) -> grpc::Result<()> {
        resp.finish(Empty::new())
    }

    fn unary_call(&self, _ctx: ServerHandlerContext, req: ServerRequestSingle<SimpleRequest>, mut resp: ServerResponseUnarySink<SimpleResponse>) -> grpc::Result<()> {
        if req.message.get_response_status().get_code() != 0 {
            return resp.send_grpc_error(
                GrpcStatus::from_code_or_unknown(req.message.response_status.get_ref().get_code() as u32),
                req.message.response_status.get_ref().message.clone(),
            );
        }

        let mut payload = Payload::new();
        payload.set_body(make_string(req.message.get_response_size() as usize));
        let mut response = SimpleResponse::new();
        response.set_payload(payload);
        resp.send_metadata(echo_custom_metadata(&req.metadata))?;
        resp.finish_with_trailers(response, echo_custom_trailing(&req.metadata))
    }

    // TODO: is this needed? I can't find it implemented in grpc-go/interop/client/client.go
    fn cacheable_unary_call(&self, _ctx: ServerHandlerContext, _req: ServerRequestSingle<SimpleRequest>, resp: ServerResponseUnarySink<SimpleResponse>) -> grpc::Result<()> {
        // TODO: implement fully
        resp.finish(SimpleResponse::new())
    }

    fn streaming_output_call(&self, o: ServerHandlerContext, mut req: ServerRequestSingle<StreamingOutputCallRequest>, resp: ServerResponseSink<StreamingOutputCallResponse>) -> grpc::Result<()> {
        let sizes = req
            .message
            .take_response_parameters()
            .into_iter()
            .map(|res| res.get_size() as usize);
        let output = stream::iter_ok(sizes).map(|size| {
            let mut response = StreamingOutputCallResponse::new();
            let mut payload = Payload::new();
            payload.set_body(make_string(size));
            response.set_payload(payload);
            response
        });
        o.pump(output, resp);
        Ok(())
    }

    fn streaming_input_call(&self, _ctx: ServerHandlerContext, req: ServerRequest<StreamingInputCallRequest>, resp: ServerResponseUnarySink<StreamingInputCallResponse>) -> grpc::Result<()> {
        let mut resp = Some(resp);
        let mut aggregate_size = 0;
        req.register_stream_handler_basic(move |message| {
            Ok(match message {
                Some(m) => {
                    aggregate_size += m.payload.get_ref().body.len() as i32;
                }
                None => {
                    let mut response = StreamingInputCallResponse::new();
                    response.set_aggregated_payload_size(aggregate_size);
                    resp.take().unwrap().finish(response)?;
                }
            })
        });
        Ok(())
    }

    fn full_duplex_call(&self, o: ServerHandlerContext, req: ServerRequest<StreamingOutputCallRequest>, mut resp: ServerResponseSink<StreamingOutputCallResponse>) -> grpc::Result<()> {
        let metadata = req.metadata();
        resp.send_metadata(echo_custom_metadata(&metadata))?;
        let mut req = req.into_stream();
        o.spawn_poll_fn(move || {
            loop {
                if let Async::NotReady = resp.poll()? {
                    return Ok(Async::NotReady);
                }
                match req.poll()? {
                    Async::Ready(Some(m)) => {
                        if m.response_status.get_ref().get_code() != 0 {
                            resp.send_grpc_error(
                                GrpcStatus::from_code_or_unknown(m.response_status.get_ref().code as u32),
                                m.response_status.get_ref().message.clone(),
                            )?;
                            return Ok(Async::Ready(()));
                        }

                        for p in &m.response_parameters {
                            resp.send_data({
                                let mut response = StreamingOutputCallResponse::new();
                                let mut payload = Payload::new();
                                payload.set_body(make_string(p.size as usize));
                                response.set_payload(payload);
                                response
                            })?;
                        }
                    }
                    Async::Ready(None) => {
                        resp.send_trailers(echo_custom_trailing(&metadata))?;
                        return Ok(Async::Ready(()));
                    }
                    Async::NotReady => {
                        return Ok(Async::NotReady)
                    }
                }
            }
        });
        Ok(())
    }

    // TODO: implement this if we find an interop client that needs it.
    fn half_duplex_call(&self, _ctx: ServerHandlerContext, _req: ServerRequest<StreamingOutputCallRequest>, mut resp: ServerResponseSink<StreamingOutputCallResponse>) -> grpc::Result<()> {
        resp.send_trailers(Metadata::new())
    }
}

fn main() {
    env_logger::init();

    let mut server = ServerBuilder::new_plain();
    server.http.set_port(DEFAULT_PORT);
    server.add_service(TestServiceServer::new_service_def(TestServerImpl {}));
    let _server = server.build().expect("server");

    loop {
        thread::park();
    }
}
