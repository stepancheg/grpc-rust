use std::iter;

use futures::stream::BoxStream;
use futures::stream;
use futures::stream::Stream;


pub fn stream_repeat<T : Clone + Send + 'static, E>(t: T) -> BoxStream<T, E> {
    let ts = iter::repeat(t).map(|t| Ok(t));
    stream::iter(ts)
        .boxed()
}

