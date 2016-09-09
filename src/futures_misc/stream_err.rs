use futures;
use futures::stream::Stream;

use super::future_to_stream_once;


pub fn stream_err<T : 'static, E : 'static>(error: E) -> Box<Stream<Item=T, Error=E>> {
    Box::new(future_to_stream_once(futures::failed(error)))
}
