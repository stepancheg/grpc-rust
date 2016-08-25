use std::sync::Mutex;

use futures::Future;
use futures::stream::channel;
use futures::stream::Sender;
use futures::stream::Receiver;
//use futures::stream::FutureSender;

#[allow(dead_code)]
pub fn channel_sync_sender<T, E>() -> (SyncSender<T, E>, Receiver<T, E>) {
    let (sender, receiver) = channel();
    (SyncSender { sender: Mutex::new(Some(sender)) }, receiver)
}

#[allow(dead_code)]
pub struct SyncSender<T, E> {
    sender: Mutex<Option<Sender<T, E>>>
}

#[allow(dead_code)]
impl<T, E> SyncSender<T, E> {
    pub fn send(&self, result: Result<T, E>) {
        let mut ptr = self.sender.lock().unwrap();
        let sender: Sender<T, E> = (*ptr).take().unwrap();
        let sender: Sender<T, E> = sender.send(result).wait().ok().unwrap();
        *ptr = Some(sender);
    }
}
