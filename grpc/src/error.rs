use std::error::Error as std_Error;
use std::fmt;
use std::io;

use httpbis;

use crate::proto::metadata;

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
    MetadataDecode(metadata::MetadataDecodeError),
    Panic(String),
    Marshaller(Box<dyn std_Error + Send + Sync>),
    Other(&'static str),
}

impl From<httpbis::SendError> for Error {
    fn from(e: httpbis::SendError) -> Self {
        Error::Http(httpbis::Error::from(e))
    }
}

impl From<httpbis::StreamDead> for Error {
    fn from(e: httpbis::StreamDead) -> Self {
        Error::Http(httpbis::Error::from(e))
    }
}

fn _assert_debug<D: ::std::fmt::Debug>(_: &D) {}

fn _assert_grpc_error_debug(e: &Error) {
    _assert_debug(e);
}

impl std_Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Io(ref err) => write!(f, "io error: {}", err),
            &Error::Http(ref err) => write!(f, "http error: {}", err),
            &Error::GrpcMessage(ref err) => write!(f, "grpc message error: {}", err.grpc_message),
            &Error::MetadataDecode(..) => write!(f, "metadata decode error"),
            &Error::Panic(ref message) => write!(f, "panic: {}", message),
            &Error::Other(ref message) => write!(f, "other error: {}", message),
            &Error::Marshaller(ref e) => write!(f, "marshaller error: {}", e),
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
    fn from(err: Error) -> httpbis::Error {
        httpbis::Error::StdError(Box::new(err))
    }
}
