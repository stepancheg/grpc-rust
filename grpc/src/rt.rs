//! Functions used by generated code, but not exposed in `grpc`.

pub use server::method::MethodHandler;
pub use server::method::MethodHandlerBidi;
pub use server::method::MethodHandlerClientStreaming;
pub use server::method::MethodHandlerServerStreaming;
pub use server::method::MethodHandlerUnary;
pub use server::method::ServerMethod;

pub use method::GrpcStreaming;
pub use method::GrpcStreamingFlavor;
pub use method::MethodDescriptor;

pub use server::ServerServiceDefinition;
