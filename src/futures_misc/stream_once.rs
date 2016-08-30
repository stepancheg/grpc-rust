use futures::stream::Stream;
use futures::BoxFuture;
use futures::Future;
use futures;


pub fn stream_once<S>(stream: S) -> BoxFuture<S::Item, S::Error>
    where
        S : Stream + Send + 'static,
        S::Item : Send + 'static,
        S::Error : Send + 'static,
{
    stream
        .fold(None, |option, item| {
            match option {
                Some(..) => panic!("more than one element"), // TODO: better error
                None => futures::finished::<_, S::Error>(Some(item))
            }
        })
        .map(|option| {
            option.expect("expecting one element, found none") // TODO: better error
        })
        .boxed()
}
