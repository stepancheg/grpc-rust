#![allow(dead_code)]

use std::mem;

use futures::stream::Stream;
use futures::task::Context;

use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

enum State<I> {
    Init,
    First(I),
    Eof,
}

pub struct StreamSingle<S>
where
    S: Stream,
{
    stream: S,
    state: State<S::Item>,
}

/// Convert a stream into single element future.
/// It is an error, is stream is empty or has more than one element.
pub fn stream_single<S>(stream: S) -> StreamSingle<S>
where
    S: Stream,
{
    StreamSingle {
        stream,
        state: State::Init,
    }
}

impl<S> Future for StreamSingle<S>
where
    S: Stream,
{
    type Output = S::Item;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let self_mut = unsafe { self.get_unchecked_mut() };
        loop {
            match unsafe { Pin::new_unchecked(&mut self_mut.stream) }.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    return match mem::replace(&mut self_mut.state, State::Eof) {
                        State::Init => panic!("expecting one element, found none"),
                        State::Eof => panic!("Future::poll after EOF"),
                        State::First(f) => Poll::Ready(f),
                    };
                }
                Poll::Ready(Some(item)) => {
                    match mem::replace(&mut self_mut.state, State::First(item)) {
                        State::Init => {}
                        State::First(_) => panic!("more than one element"),
                        State::Eof => panic!("poll after EOF"),
                    }
                }
            }
        }
    }
}
