use std::marker;
use std::mem;

use futures::stream::Stream;
use futures::Future;
use futures::Async;
use futures::Poll;


#[must_use = "streams do nothing unless polled"]
pub struct FutureFlattenToStream<F>
    where F: Future,
          <F as Future>::Item: Stream<Error=F::Error>,
{
    state: State<F>
}

pub fn future_flatten_to_stream<F>(f: F) -> FutureFlattenToStream<F>
    where
        F : Future,
        <F as Future>::Item: Stream<Error=F::Error>,
{
    FutureFlattenToStream {
        state: State::Future(f)
    }
}

/// Stream that emits single error
enum FailedState<T, E> {
    Error(E, marker::PhantomData<T>),
    Eof,
    Done,
}

impl<T, E> Stream for FailedState<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match mem::replace(self, FailedState::Done) {
            FailedState::Done => panic!("cannot poll after eof"),
            FailedState::Eof => {
                *self = FailedState::Done;
                Ok(Async::Ready(None))
            }
            FailedState::Error(e, _) => {
                *self = FailedState::Eof;
                Err(e)
            }
        }
    }
}

enum State<F>
    where F: Future,
          <F as Future>::Item: Stream<Error=F::Error>,
{
    // future is not yet called or called and not ready
    Future(F),
    // future resolved to Stream
    Stream(F::Item),
    // future resolved to error
    Failed(FailedState<<F::Item as Stream>::Item, <F::Item as Stream>::Error>),
}

impl<F> Stream for FutureFlattenToStream<F>
    where F: Future,
          <F as Future>::Item: Stream<Error=F::Error>,
{
    type Item = <F::Item as Stream>::Item;
    type Error = <F::Item as Stream>::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            self.state = match self.state {
                State::Future(ref mut f) => {
                    match f.poll() {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(stream)) => {
                            State::Stream(stream)
                        }
                        Err(e) => {
                            State::Failed(
                                FailedState::Error(e, marker::PhantomData))
                        }
                    }
                }
                State::Stream(ref mut s) => {
                    return s.poll();
                }
                State::Failed(ref mut s) => {
                    return s.poll();
                }
            };
        }
    }
}
