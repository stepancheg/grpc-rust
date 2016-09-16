use std::iter;

use futures::stream;


#[allow(dead_code)]
pub fn stream_once<T, E>(t: T) -> stream::IterStream<iter::Once<Result<T, E>>> {
    let ts = iter::once(Ok(t));
    stream::iter(ts)
}
