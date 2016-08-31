use std::iter;

use futures::stream::BoxStream;
use futures::stream;
use futures::stream::Stream;


#[allow(dead_code)]
pub fn stream_once<T : Clone + Send + 'static, E>(t: T) -> BoxStream<T, E> {
    let ts = iter::once(t).map(|t| Ok(t));
    stream::iter(ts)
        .boxed()
}

