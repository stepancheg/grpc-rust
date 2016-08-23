use std::io;


#[derive(Debug)]
pub enum GrpcError {
    Io(io::Error),
    Other,
}

pub type GrpcResult<T> = Result<T, GrpcError>;


impl From<io::Error> for GrpcError {
    fn from(err: io::Error) -> Self {
        GrpcError::Io(err)
    }
}
