use std::marker;

use futures::Async;
use futures::Poll;
use futures::stream::Stream;


pub struct StreamRepeat<T : Clone, E>(T, marker::PhantomData<E>);

pub fn stream_repeat<T : Clone, E>(t: T) -> StreamRepeat<T, E> {
    StreamRepeat(t, marker::PhantomData)
}

impl<T : Clone, E> Stream for StreamRepeat<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(Some(self.0.clone())))
    }
}
