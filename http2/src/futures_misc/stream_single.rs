use futures::stream::Stream;
use futures::Future;
use futures::Poll;
use futures::Async;


pub struct StreamSingle<S>
    where S : Stream
{
    stream: S,
    state: Option<S::Item>,
}


/// Convert a stream into single element future.
/// It is an error, is stream is empty or has more than one element.
pub fn stream_single<S>(stream: S) -> StreamSingle<S>
    where S : Stream
{
    StreamSingle {
        stream: stream,
        state: None,
    }
}

impl<S> Future for StreamSingle<S>
    where S : Stream
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match try_ready!(self.stream.poll()) {
                // TODO: better errors
                None => return Ok(Async::Ready(self.state.take().expect("expecting one element, found none"))),
                Some(item) => {
                    if self.state.is_some() {
                        panic!("more than one element");
                    }
                    self.state = Some(item);
                }
            }
        }
    }
}

