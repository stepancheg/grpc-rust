use std::io;

use futures::Future;
use futures::Poll;
use futures::Async;
use futures::stream::Stream;
use futures;


#[allow(dead_code)]
pub fn oneshot<T : Send + 'static>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = futures::sync::mpsc::unbounded();
    (Sender { sender: sender }, Receiver { receiver: Some(receiver) })
}

#[allow(dead_code)]
pub struct Sender<T> {
    sender: futures::sync::mpsc::UnboundedSender<T>,
}

#[allow(dead_code)]
pub struct Receiver<T> {
    receiver: Option<futures::sync::mpsc::UnboundedReceiver<T>>,
}

#[allow(dead_code)]
impl<T> Sender<T> {
    pub fn send(self, t: T) -> Result<(), futures::sync::mpsc::SendError<T>> {
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
                match try_ready!(receiver.poll().map_err(|_| io::Error::new(io::ErrorKind::Other, "recv"))) {
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
