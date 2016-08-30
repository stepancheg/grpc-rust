use futures;
use futures::stream::BoxStream;
use futures::stream::Stream;

use super::future_to_stream_once;


pub fn stream_err<T : Send + 'static, E : Send + 'static>(error: E) -> BoxStream<T, E> {
    future_to_stream_once(futures::failed(error))
        .boxed()
}
