use std::sync::Arc;

mod test_sync;
mod stream_thread_spawn_iter;

use grpc::method::GrpcStreaming;
use grpc::method::MethodDescriptor;
use grpc::marshall::MarshallerString;

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
