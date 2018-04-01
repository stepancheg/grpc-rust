#![allow(dead_code)]

use std::thread;

use futures;
use futures::stream::Stream;


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
    let (sender, receiver) = futures::sync::mpsc::unbounded();
    thread::spawn(move || {
        for item in f() {
            sender.unbounded_send(item).expect("send");
        }
    });

    let receiver = receiver.then(|r| {
        match r {
            Ok(r) => Ok(r),
            Err(()) => panic!("unexpected"),
        }
    });

    Box::new(receiver)
}
