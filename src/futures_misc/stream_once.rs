use std::iter;

use futures::stream;
use futures::stream::Stream;


#[allow(dead_code)]
pub fn stream_once<T : 'static, E>(t: T) -> Box<Stream<Item=T, Error=E>> {
    let ts = iter::once(t).map(|t| Ok(t));
    Box::new(stream::iter(ts))
}

#[allow(dead_code)]
pub fn stream_once_send<T : Send + 'static, E>(t: T) -> Box<Stream<Item=T, Error=E> + Send> {
    let ts = iter::once(t).map(|t| Ok(t));
    Box::new(stream::iter(ts))
}

