use std::sync::Arc;

mod stream_thread_spawn_iter;
mod test_sync;

use grpc::for_test::*;
use grpc::rt::*;

pub use self::stream_thread_spawn_iter::stream_thread_spawn_iter;
pub use self::test_sync::TestSync;

use log_ndc_env_logger;
use std::sync::Once;

pub fn string_string_method(
    name: &str,
    streaming: GrpcStreaming,
) -> ArcOrStatic<MethodDescriptor<String, String>> {
    ArcOrStatic::Arc(Arc::new(MethodDescriptor {
        name: name.into(),
        streaming,
        req_marshaller: ArcOrStatic::Static(&MarshallerString),
        resp_marshaller: ArcOrStatic::Static(&MarshallerString),
    }))
}

// Bind on IPv4 because IPv6 is broken on travis
pub const BIND_HOST: &str = "127.0.0.1";

pub fn init_logger() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        log_ndc_env_logger::init();
    });
}
