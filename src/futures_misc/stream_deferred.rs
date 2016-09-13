use std::convert::From;
use std::io;

use futures::Future;
use futures::Poll;
use futures::stream::Stream;

use tokio_oneshot;
use tokio_core::reactor;


enum State<S> {
    Receiver(tokio_oneshot::Receiver<S>),
    Ready(S),
}


#[allow(dead_code)]
pub struct Deferred<S>
    where
        S : Stream + Send + 'static,
        S::Error : From<io::Error>,
{
    state: State<S>,
}

#[allow(dead_code)]
pub struct DeferredInit<S>
    where
        S : Stream + Send + 'static,
        S::Error : From<io::Error>,
{
    sender: tokio_oneshot::Sender<S>,
}

/// Stream with late init.
#[allow(dead_code)]
pub fn stream_deferred<S>(handle: &reactor::Handle) -> (Deferred<S>, DeferredInit<S>)
    where
        S : Stream + Send + 'static,
        S::Error : From<io::Error>,
{
    let (tx, rx) = tokio_oneshot::oneshot(handle);

    (
        Deferred { state: State::Receiver(rx) },
        DeferredInit { sender: tx }
    )
}

impl<S> DeferredInit<S>
    where
        S : Stream + Send + 'static,
        S::Error : From<io::Error>,
{
    pub fn init(self, stream: S) -> io::Result<()> {
        self.sender.send(stream)
    }
}

impl<S> Stream for Deferred<S>
    where
        S : Stream + Send + 'static,
        S::Error : From<io::Error>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let stream: S = match self.state {
            State::Ready(ref mut s) => return s.poll(),
            State::Receiver(ref mut receiver) => try_ready!(receiver.poll()),
        };

        self.state = State::Ready(stream);
        match self.state {
            State::Ready(ref mut s) => s.poll(),
            State::Receiver(..) => unreachable!(),
        }
    }
}
