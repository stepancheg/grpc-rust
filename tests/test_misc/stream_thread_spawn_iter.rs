#![allow(dead_code)]

use std::thread;

use futures::stream::Stream;

use grpc::futures_misc::channel_sync_sender;

/// Spawn a thread with a function which returns an iterator.
/// Resulting iterator elements will be emitted as Stream.
pub fn stream_thread_spawn_iter<I, F, E>(f: F)
    -> Box<Stream<Item=I::Item, Error=E> + Send>
        where
            I : Iterator,
            F : FnOnce() -> I,
            F : Send + 'static,
            E : Send + 'static,
            I::Item : Send + 'static,
{
    let (sender, receiver) = channel_sync_sender();
    thread::spawn(move || {
        for item in f() {
            sender.send(Ok(item));
        }
    });

    Box::new(receiver)
}
