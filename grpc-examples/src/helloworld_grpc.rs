// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


pub trait Greeter {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> super::helloworld::HelloReply;
}

pub struct GreeterClient {
}

impl Greeter for GreeterClient {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> super::helloworld::HelloReply {
        panic!("TODO: not yet");
    }
}

pub struct GreeterServer {
    h: Box<Greeter>,
}

impl GreeterServer {
    pub fn new<H : Greeter + 'static>(h: H) -> GreeterServer {
        ::grpc::method::ServerServiceDefinition::new(
            vec![
                ::grpc::method::ServerMethod::new(
                    ::grpc::method::MethodDescriptor {
                        name: "/helloworld.Greeter/SayHello".to_string(),
                        input_streaming: false,
                        output_streaming: false,
                        req_marshaller: Box::new(::grpc::marshall::MarshallerBytes),
                        resp_marshaller: Box::new(::grpc::marshall::MarshallerBytes),
                    },
                    Box::new(::grpc::method::MethodHandlerEcho),
                ),
            ],
        );
        GreeterServer {
            h: Box::new(h),
        }
    }
}
