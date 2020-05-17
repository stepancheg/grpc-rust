#[macro_use]
extern crate log;

use grpc_interop::*;

use std::thread;

use bytes::Bytes;

use futures::stream;
use futures::StreamExt;

use grpc::*;
use std::task::Poll;

static DICTIONARY: &'static str = "ABCDEFGHIJKLMNOPQRSTUVabcdefghijklmnoqprstuvwxyz0123456789";

/**
 * Returns a Vec<u8> with a given size containing printable u8s.
 */
fn make_string(size: usize) -> Vec<u8> {
    let dict = DICTIONARY.to_owned().into_bytes();
    let mut result = Vec::<u8>::with_capacity(size);

    for n in 0..size {
        result.push(dict[n % DICTIONARY.len()]);
    }

    return result;
}

fn echo_custom_metadata(req_metadata: &Metadata) -> Metadata {
    static TEST_ECHO_KEY: &'static str = "x-grpc-test-echo-initial";
    let mut metadata = Metadata::new();
    if let Some(value) = req_metadata.get(TEST_ECHO_KEY) {
        metadata.add(
            MetadataKey::from(TEST_ECHO_KEY),
            Bytes::copy_from_slice(value),
        );
    }
    metadata
}

fn echo_custom_trailing(req_metadata: &Metadata) -> Metadata {
    static TEST_ECHO_TRAILING_KEY: &'static str = "x-grpc-test-echo-trailing-bin";
    let mut metadata = Metadata::new();
    if let Some(value) = req_metadata.get(TEST_ECHO_TRAILING_KEY) {
        metadata.add(
            MetadataKey::from(TEST_ECHO_TRAILING_KEY),
            Bytes::copy_from_slice(value),
        );
    }
    metadata
}

struct TestServerImpl {}

impl TestService for TestServerImpl {
    // One empty request followed by one empty response.
    fn empty_call(
        &self,
        _ctx: ServerHandlerContext,
        _req: ServerRequestSingle<Empty>,
        resp: ServerResponseUnarySink<Empty>,
    ) -> grpc::Result<()> {
        resp.finish(Empty::new())
    }

    // One request followed by one response.
    fn unary_call(
        &self,
        _ctx: ServerHandlerContext,
        req: ServerRequestSingle<SimpleRequest>,
        mut resp: ServerResponseUnarySink<SimpleResponse>,
    ) -> grpc::Result<()> {
        if req.message.get_response_status().get_code() != 0 {
            debug!(
                "requested to send grpc error {}",
                req.message.get_response_status().get_code()
            );
            return resp.send_grpc_error(
                GrpcStatus::from_code_or_unknown(
                    req.message.get_response_status().get_code() as u32
                ),
                req.message.get_response_status().message.clone(),
            );
        }

        let mut payload = Payload::new();
        payload.set_body(make_string(req.message.get_response_size() as usize));
        let mut response = SimpleResponse::new();
        response.set_payload(payload);
        resp.send_metadata(echo_custom_metadata(&req.metadata))?;
        resp.finish_with_trailers(response, echo_custom_trailing(&req.metadata))
    }

    // One request followed by one response. Response has cache control
    // headers set such that a caching HTTP proxy (such as GFE) can
    // satisfy subsequent requests.
    fn cacheable_unary_call(
        &self,
        _ctx: ServerHandlerContext,
        _req: ServerRequestSingle<SimpleRequest>,
        resp: ServerResponseUnarySink<SimpleResponse>,
    ) -> grpc::Result<()> {
        // TODO: implement fully
        resp.finish(SimpleResponse::new())
    }

    // One request followed by a sequence of responses (streamed download).
    // The server returns the payload with client desired type and sizes.
    fn streaming_output_call(
        &self,
        o: ServerHandlerContext,
        mut req: ServerRequestSingle<StreamingOutputCallRequest>,
        resp: ServerResponseSink<StreamingOutputCallResponse>,
    ) -> grpc::Result<()> {
        let sizes = req
            .message
            .take_response_parameters()
            .into_iter()
            .map(|res| res.get_size() as usize);
        let output = stream::iter(sizes).map(|size| {
            let mut response = StreamingOutputCallResponse::new();
            let mut payload = Payload::new();
            payload.set_body(make_string(size));
            response.set_payload(payload);
            Ok(response)
        });
        o.pump(output, resp);
        Ok(())
    }

    // A sequence of requests followed by one response (streamed upload).
    // The server returns the aggregated size of client payload as the result.
    fn streaming_input_call(
        &self,
        _ctx: ServerHandlerContext,
        req: ServerRequest<StreamingInputCallRequest>,
        resp: ServerResponseUnarySink<StreamingInputCallResponse>,
    ) -> grpc::Result<()> {
        let mut resp = Some(resp);
        let mut aggregate_size = 0;
        req.register_stream_handler_basic(move |message| {
            Ok(match message {
                Some(m) => {
                    aggregate_size += m.get_payload().body.len() as i32;
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

    // A sequence of requests with each request served by the server immediately.
    // As one request could lead to multiple responses, this interface
    // demonstrates the idea of full duplexing.
    fn full_duplex_call(
        &self,
        o: ServerHandlerContext,
        req: ServerRequest<StreamingOutputCallRequest>,
        mut resp: ServerResponseSink<StreamingOutputCallResponse>,
    ) -> grpc::Result<()> {
        let metadata = req.metadata();
        debug!("sending custom metadata");
        resp.send_metadata(echo_custom_metadata(&metadata))?;
        let mut req = req.into_stream();
        o.spawn_poll_fn(move |cx| loop {
            if let Poll::Pending = resp.poll(cx)? {
                return Poll::Pending;
            }
            match req.poll_next_unpin(cx)? {
                Poll::Ready(Some(m)) => {
                    if m.get_response_status().get_code() != 0 {
                        debug!(
                            "requested to send grpc error {}",
                            m.get_response_status().get_code()
                        );
                        resp.send_grpc_error(
                            GrpcStatus::from_code_or_unknown(m.get_response_status().code as u32),
                            m.get_response_status().message.clone(),
                        )?;
                        return Poll::Ready(Ok(()));
                    }

                    for p in &m.response_parameters {
                        debug!("requested to send data of size {}", p.size);
                        resp.send_data({
                            let mut response = StreamingOutputCallResponse::new();
                            let mut payload = Payload::new();
                            payload.set_body(make_string(p.size as usize));
                            response.set_payload(payload);
                            response
                        })?;
                    }
                }
                Poll::Ready(None) => {
                    debug!("sending custom trailers");
                    resp.send_trailers(echo_custom_trailing(&metadata))?;
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => return Poll::Pending,
            }
        });
        Ok(())
    }

    // TODO: implement this if we find an interop client that needs it.
    // A sequence of requests followed by a sequence of responses.
    // The server buffers all the client requests and then serves them in order. A
    // stream of responses are returned to the client when the server starts with
    // first request.
    fn half_duplex_call(
        &self,
        _ctx: ServerHandlerContext,
        _req: ServerRequest<StreamingOutputCallRequest>,
        mut resp: ServerResponseSink<StreamingOutputCallResponse>,
    ) -> grpc::Result<()> {
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
