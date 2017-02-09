pub static HEADER_GRPC_STATUS: &'static str = "grpc-status";
pub static HEADER_GRPC_MESSAGE: &'static str = "grpc-message";

// gRPC status codes.
// TODO: more status codes.
// TODO: should we move it to another mod?

/// Not an error; returned on success.
pub const OK: u32 = 0;
