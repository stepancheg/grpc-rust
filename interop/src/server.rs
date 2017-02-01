extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate env_logger;

pub mod empty;
pub mod messages;
pub mod test_grpc;

use std::thread;

use futures::stream::Stream;
use futures::stream;
use futures::Future;

use empty::*;
use messages::{Payload, SimpleRequest, SimpleResponse, StreamingOutputCallRequest, 
  StreamingOutputCallResponse, StreamingInputCallRequest, StreamingInputCallResponse};
use test_grpc::*;

use grpc::futures_grpc::{GrpcFutureSend, GrpcStreamSend};
use grpc::error::GrpcError;

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
    fn EmptyCall(&self, _: Empty) -> GrpcFutureSend<Empty> {
        Box::new(futures::finished(Empty::new()))
    }

    fn UnaryCall(&self, req: SimpleRequest) -> GrpcFutureSend<SimpleResponse> {
        let mut payload = Payload::new();
        payload.set_body(make_string(req.get_response_size() as usize));
        let mut response = SimpleResponse::new();
        response.set_payload(payload);
        Box::new(futures::finished(response))
    }

    // TODO: is this needed? I can't find it implemented in grpc-go/interop/client/client.go
    fn CacheableUnaryCall(&self, _: SimpleRequest) -> GrpcFutureSend<SimpleResponse> {
        // TODO: implement fully
        Box::new(futures::finished(SimpleResponse::new()))
    }

    fn StreamingOutputCall(&self, mut req: StreamingOutputCallRequest) -> GrpcStreamSend<StreamingOutputCallResponse> {
        let sizes = req.take_response_parameters().into_iter().map(|res| Ok(res.get_size() as usize));
        let output = stream::iter(sizes).map(|size| {
            let mut response = StreamingOutputCallResponse::new();
            let mut payload = Payload::new();
            payload.set_body(make_string(size));
            response.set_payload(payload);
            response
        });
        Box::new(output)
    }

    fn StreamingInputCall(&self, req_stream: GrpcStreamSend<StreamingInputCallRequest>) -> GrpcFutureSend<StreamingInputCallResponse> {
        let return_stream = req_stream
            .map(|req| req.get_payload().body.len() as i32)
            .fold(0, |a, b| futures::finished::<_, GrpcError>(a + b))
            .map(|aggregate_size| {
                let mut response = StreamingInputCallResponse::new();
                response.set_aggregated_payload_size(aggregate_size);
                response
            });
        Box::new(return_stream)
    }

    fn FullDuplexCall(&self, req_stream: GrpcStreamSend<StreamingOutputCallRequest>) -> GrpcStreamSend<StreamingOutputCallResponse> {
        let response = req_stream.map(|mut req| {
            let sizes = req.take_response_parameters().into_iter().map(|res| Ok(res.get_size() as usize));
            stream::iter(sizes).map(|size| {
                let mut response = StreamingOutputCallResponse::new();
                let mut payload = Payload::new();
                payload.set_body(make_string(size));
                response.set_payload(payload);
                response
            })
        }).flatten();
        Box::new(response)
    }

    // TODO: implement this if we find an interop client that needs it.
    fn HalfDuplexCall(&self, _: GrpcStreamSend<StreamingOutputCallRequest>) -> GrpcStreamSend<StreamingOutputCallResponse> {
        Box::new(stream::empty())
    }
}

fn main() {
    drop(env_logger::init().unwrap());

    let _server = TestServiceAsyncServer::new("[::]:60011", Default::default(), TestServerImpl {});

    loop {
        thread::park();
    }
}
