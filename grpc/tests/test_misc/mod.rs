use std::sync::Arc;

mod test_sync;
mod stream_thread_spawn_iter;

use grpc::rt::*;
use grpc::for_test::*;

pub use self::test_sync::TestSync;
pub use self::stream_thread_spawn_iter::stream_thread_spawn_iter;


pub fn string_string_method(name: &str, streaming: GrpcStreaming)
    -> Arc<MethodDescriptor<String, String>>
{
    Arc::new(MethodDescriptor {
       name: name.to_owned(),
       streaming: streaming,
       req_marshaller: Box::new(MarshallerString),
       resp_marshaller: Box::new(MarshallerString),
   })
}

// Bind on IPv4 because IPv6 is broken on travis
pub const BIND_HOST: &str = "127.0.0.1";
