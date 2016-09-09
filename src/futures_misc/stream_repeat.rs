use std::iter;

use futures::stream;
use futures::stream::Stream;


pub fn stream_repeat<T : Clone + 'static, E>(t: T) -> Box<Stream<Item=T, Error=E>> {
    let ts = iter::repeat(t).map(|t| Ok(t));
    Box::new(stream::iter(ts))
}

