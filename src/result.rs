use std::io;

pub enum GrpcError {
    Io(io::Error),
    Other,
}

pub type GrpcResult<T> = Result<T, GrpcError>;
