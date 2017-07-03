//! Functions used by generated code, but not exposed in `grpc`.

pub use server_method::ServerMethod;
pub use server_method::MethodHandler;
pub use server_method::MethodHandlerUnary;
pub use server_method::MethodHandlerClientStreaming;
pub use server_method::MethodHandlerServerStreaming;
pub use server_method::MethodHandlerBidi;

pub use method::GrpcStreaming;
pub use method::GrpcStreamingFlavor;
pub use method::MethodDescriptor;

pub use server::ServerServiceDefinition;
