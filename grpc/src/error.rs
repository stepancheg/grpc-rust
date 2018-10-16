use std::error::Error as StdError;
use std::fmt;
use std::io;

use futures;

use metadata;

use httpbis;

use protobuf_lib::ProtobufError;

#[derive(Debug)]
pub struct GrpcMessageError {
    pub grpc_status: i32,

    /// Content of `grpc-message` header
    pub grpc_message: String,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Http(httpbis::Error),
    GrpcMessage(GrpcMessageError),
    Canceled(futures::Canceled),
    MetadataDecode(metadata::MetadataDecodeError),
    Protobuf(ProtobufError),
    Panic(String),
    Other(&'static str),
}

fn _assert_debug<D: ::std::fmt::Debug>(_: &D) {}

fn _assert_grpc_error_debug(e: &Error) {
    _assert_debug(e);
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref err) => err.description(),
            &Error::Http(ref err) => err.description(),
            &Error::GrpcMessage(ref err) => &err.grpc_message,
            &Error::MetadataDecode(..) => "metadata decode error",
            &Error::Protobuf(ref err) => err.description(),
            &Error::Canceled(..) => "canceled",
            &Error::Panic(ref message) => &message,
            &Error::Other(ref message) => message,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => write!(f, "io error: {}", err.description()),
            &Error::Http(ref err) => write!(f, "http error: {}", err.description()),
            &Error::GrpcMessage(ref err) => write!(f, "grpc message error: {}", err.grpc_message),
            &Error::MetadataDecode(..) => write!(f, "metadata decode error"),
            &Error::Protobuf(ref err) => write!(f, "protobuf error: {}", err.description()),
            &Error::Canceled(..) => write!(f, "canceled"),
            &Error::Panic(ref message) => write!(f, "panic: {}", message),
            &Error::Other(ref message) => write!(f, "other error: {}", message),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<httpbis::Error> for Error {
    fn from(err: httpbis::Error) -> Self {
        Error::Http(err)
    }
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::Protobuf(err)
    }
}

impl From<futures::Canceled> for Error {
    fn from(err: futures::Canceled) -> Self {
        Error::Canceled(err)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        match err {
            Error::Io(e) => e,
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}

impl From<metadata::MetadataDecodeError> for Error {
    fn from(de: metadata::MetadataDecodeError) -> Self {
        Error::MetadataDecode(de)
    }
}

impl From<Error> for httpbis::Error {
    fn from(_err: Error) -> httpbis::Error {
        httpbis::Error::Other("grpc error") // TODO: preserve
    }
}
