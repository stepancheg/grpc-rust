use std::io;

use futures::Future;
use futures::Poll;
use futures::Async;
use futures::stream::Stream;

use tokio_core;
use tokio_core::reactor;


#[allow(dead_code)]
pub fn oneshot<T : Send + 'static>(handle: &reactor::Handle) -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = tokio_core::channel::channel(handle).unwrap();
    (Sender { sender: sender }, Receiver { receiver: Some(receiver) })
}

#[allow(dead_code)]
pub struct Sender<T> {
    sender: tokio_core::channel::Sender<T>,
}

#[allow(dead_code)]
pub struct Receiver<T> {
    receiver: Option<tokio_core::channel::Receiver<T>>,
}

#[allow(dead_code)]
impl<T> Sender<T> {
    pub fn send(self, t: T) -> io::Result<()> {
        self.sender.send(t)
    }
}

impl<T> Future for Receiver<T> {
    type Item = T;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let t = match &mut self.receiver {
            &mut None => panic!("cannot be polled twice"),
            &mut Some(ref mut receiver) => {
                match try_ready!(receiver.poll()) {
                    Some(t) => t,
                    None =>
                        return Err(io::Error::new(io::ErrorKind::Other,
                            "channel has been disconnected")),
                }
            }
        };

        self.receiver = None;

        Ok(Async::Ready(t))
    }
}
