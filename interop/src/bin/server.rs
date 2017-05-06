extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate env_logger;

extern crate grpc_interop;
use grpc_interop::*;

use std::thread;

use futures::stream::Stream;
use futures::stream;
use futures::Future;

use grpc::futures_grpc::*;
use grpc::error::*;
use grpc::*;

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
        result.push(dict[n%DICTIONARY_SIZE]);
    }

    return result;
}

struct TestServerImpl {}

impl TestServiceAsync for TestServerImpl {
    fn EmptyCall(&self, _: Empty) -> GrpcSingleResponse<Empty> {
        GrpcSingleResponse::completed(Empty::new())
    }

    fn UnaryCall(&self, mut req: SimpleRequest) -> GrpcSingleResponse<SimpleResponse> {
        if req.get_response_status().get_code() != 0 {
            return GrpcSingleResponse::no_metadata(futures::failed(GrpcError::GrpcMessage(GrpcMessageError {
                grpc_status: req.get_response_status().get_code(),
                grpc_message: req.mut_response_status().take_message(),
            })));
        }

        let mut payload = Payload::new();
        payload.set_body(make_string(req.get_response_size() as usize));
        let mut response = SimpleResponse::new();
        response.set_payload(payload);
        GrpcSingleResponse::completed(response)
    }

    // TODO: is this needed? I can't find it implemented in grpc-go/interop/client/client.go
    fn CacheableUnaryCall(&self, _: SimpleRequest) -> GrpcSingleResponse<SimpleResponse> {
        // TODO: implement fully
        GrpcSingleResponse::completed(SimpleResponse::new())
    }

    fn StreamingOutputCall(&self, mut req: StreamingOutputCallRequest) -> GrpcStreamingResponse<StreamingOutputCallResponse> {
        let sizes = req.take_response_parameters().into_iter().map(|res| Ok(res.get_size() as usize));
        let output = stream::iter(sizes).map(|size| {
            let mut response = StreamingOutputCallResponse::new();
            let mut payload = Payload::new();
            payload.set_body(make_string(size));
            response.set_payload(payload);
            response
        });
        GrpcStreamingResponse::no_metadata(output)
    }

    fn StreamingInputCall(&self, req_stream: GrpcStreamSend<StreamingInputCallRequest>) -> GrpcSingleResponse<StreamingInputCallResponse> {
        let return_stream = req_stream
            .map(|req| req.get_payload().body.len() as i32)
            .fold(0, |a, b| futures::finished::<_, GrpcError>(a + b))
            .map(|aggregate_size| {
                let mut response = StreamingInputCallResponse::new();
                response.set_aggregated_payload_size(aggregate_size);
                response
            });
        GrpcSingleResponse::no_metadata(return_stream)
    }

    fn FullDuplexCall(&self, req_stream: GrpcStreamSend<StreamingOutputCallRequest>)
        -> GrpcStreamingResponse<StreamingOutputCallResponse>
    {
        let response = req_stream.map(|mut req| {
            if req.get_response_status().get_code() != 0 {
                let s: GrpcStreamSend<StreamingOutputCallResponse> = Box::new(stream::once(Err(GrpcError::GrpcMessage(GrpcMessageError {
                    grpc_status: req.get_response_status().get_code(),
                    grpc_message: req.mut_response_status().take_message(),
                }))));
                return s;
            }

            let sizes = req.take_response_parameters().into_iter().map(|res| Ok(res.get_size() as usize));
            let ss: GrpcStreamSend<StreamingOutputCallResponse> = Box::new(stream::iter(sizes).map(|size| {
                let mut response = StreamingOutputCallResponse::new();
                let mut payload = Payload::new();
                payload.set_body(make_string(size));
                response.set_payload(payload);
                response
            }));
            ss
        }).flatten();
        GrpcStreamingResponse::no_metadata(response)
    }

    // TODO: implement this if we find an interop client that needs it.
    fn HalfDuplexCall(&self, _: GrpcStreamSend<StreamingOutputCallRequest>) -> GrpcStreamingResponse<StreamingOutputCallResponse> {
        GrpcStreamingResponse::no_metadata(stream::empty())
    }
}

fn main() {
    env_logger::init().expect("env_logger::init");

    let _server = TestServiceAsyncServer::new(("::", DEFAULT_PORT), Default::default(), TestServerImpl {});

    loop {
        thread::park();
    }
}
