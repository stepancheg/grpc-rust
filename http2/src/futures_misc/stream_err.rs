use std::mem;
use std::marker;

use futures::Async;
use futures::Poll;
use futures::stream::Stream;


enum State<T, E> {
    Error(E, marker::PhantomData<T>),
    Eof,
    Done,
}

pub struct StreamErr<T, E> {
    state: State<T, E>,
}

pub fn stream_err<T, E>(error: E) -> StreamErr<T, E> {
    StreamErr {
        state: State::Error(error, marker::PhantomData)
    }
}


impl<T, E> Stream for StreamErr<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let (state, r) = match mem::replace(&mut self.state, State::Done) {
            State::Error(e, _) => (State::Eof, Err(e)),
            State::Eof => (State::Done, Ok(Async::Ready(None))),
            State::Done => panic!("cannot poll after eof"),
        };
        self.state = state;
        r
    }
}
