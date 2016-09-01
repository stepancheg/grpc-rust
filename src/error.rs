use std::io;
use std::error::Error;
use std::fmt;

use futures;

use solicit::http::HttpError;

use protobuf::ProtobufError;

#[derive(Debug)]
pub struct GrpcMessageError {
    /// Content of `grpc-message` header
    pub grpc_message: String,
}


#[derive(Debug)]
pub enum GrpcError {
    Io(io::Error),
    Http(HttpError),
    GrpcMessage(GrpcMessageError),
    Canceled(futures::Canceled),
    Protobuf(ProtobufError),
    Panic(String),
    Other(&'static str),
}

fn _assert_debug<D : ::std::fmt::Debug>(_: &D) {}

fn _assert_grpc_error_debug(e: &GrpcError) {
    _assert_debug(e);
}

impl Error for GrpcError {
    fn description(&self) -> &str {
        match self {
            &GrpcError::Io(ref err) => err.description(),
            &GrpcError::Http(ref err) => err.description(),
            &GrpcError::GrpcMessage(ref err) => &err.grpc_message,
            &GrpcError::Protobuf(ref err) => err.description(),
            &GrpcError::Canceled(..) => "canceled",
            &GrpcError::Panic(ref message) => &message,
            &GrpcError::Other(ref message) => message,
        }
    }
}

impl fmt::Display for GrpcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &GrpcError::Io(ref err) => write!(f, "io error: {}", err.description()),
            &GrpcError::Http(ref err) => write!(f, "http error: {}", err.description()),
            &GrpcError::GrpcMessage(ref err) => write!(f, "grpc message error: {}", err.grpc_message),
            &GrpcError::Protobuf(ref err) => write!(f, "protobuf error: {}", err.description()),
            &GrpcError::Canceled(..) => write!(f, "canceled"),
            &GrpcError::Panic(ref message) => write!(f, "panic: {}", message),
            &GrpcError::Other(ref message) => write!(f, "other error: {}", message),
        }
    }
}


impl From<io::Error> for GrpcError {
    fn from(err: io::Error) -> Self {
        GrpcError::Io(err)
    }
}

impl From<HttpError> for GrpcError {
    fn from(err: HttpError) -> Self {
        GrpcError::Http(err)
    }
}

impl From<ProtobufError> for GrpcError {
    fn from(err: ProtobufError) -> Self {
        GrpcError::Protobuf(err)
    }
}

impl From<futures::Canceled> for GrpcError {
    fn from(err: futures::Canceled) -> Self {
        GrpcError::Canceled(err)
    }
}

impl From<GrpcError> for io::Error {
    fn from(err: GrpcError) -> io::Error {
        match err {
            GrpcError::Io(e) => e,
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}


