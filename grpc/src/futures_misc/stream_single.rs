use std::mem;

use futures::future::Future;
use futures::stream::Stream;
use futures::Async;
use futures::Poll;

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
        stream: stream,
        state: State::Init,
    }
}

impl<S> Future for StreamSingle<S>
where
    S: Stream,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match try_ready!(self.stream.poll()) {
                None => {
                    return match mem::replace(&mut self.state, State::Eof) {
                        State::Init => panic!("expecting one element, found none"),
                        State::Eof => panic!("Future::poll after EOF"),
                        State::First(f) => Ok(Async::Ready(f)),
                    };
                }
                Some(item) => match mem::replace(&mut self.state, State::First(item)) {
                    State::Init => {}
                    State::First(_) => panic!("more than one element"),
                    State::Eof => panic!("poll after EOF"),
                },
            }
        }
    }
}
