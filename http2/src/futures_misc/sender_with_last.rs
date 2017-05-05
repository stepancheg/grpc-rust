use futures::sync::mpsc::SendError;
use futures::sync::mpsc::UnboundedSender;

pub struct UnboundedSenderWithFinal<T> {
    sender: UnboundedSender<T>,
    last: Option<T>,
}

impl<T> UnboundedSenderWithFinal<T> {
    pub fn new(sender: UnboundedSender<T>, last: T) -> UnboundedSenderWithFinal<T> {
        UnboundedSenderWithFinal {
            sender: sender,
            last: Some(last),
        }
    }

    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        self.sender.send(msg)
    }

    pub fn cancel_last(&mut self) {
        self.last.take();
    }
}

impl<T> Drop for UnboundedSenderWithFinal<T> {
    fn drop(&mut self) {
        if let Some(last) = self.last.take() {
            drop(self.sender.send(last));
        }
    }
}
