//! Functions used by generated code, but not exposed in `grpc`.

pub use crate::server::method::MethodHandler;
pub use crate::server::method::MethodHandlerBidi;
pub use crate::server::method::MethodHandlerClientStreaming;
pub use crate::server::method::MethodHandlerServerStreaming;
pub use crate::server::method::MethodHandlerUnary;
pub use crate::server::method::ServerMethod;

pub use crate::method::GrpcStreaming;
pub use crate::method::GrpcStreamingFlavor;
pub use crate::method::MethodDescriptor;

pub use crate::or_static::arc::ArcOrStatic;
pub use crate::or_static::string::StringOrStatic;
pub use crate::server::ServerServiceDefinition;
