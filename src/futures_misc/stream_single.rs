use futures::stream::Stream;
use futures::BoxFuture;
use futures::Future;
use futures;


/// Convert a stream into single element future.
/// It is an error, is stream is empty or has more than one element.
pub fn stream_single<S>(stream: S) -> BoxFuture<S::Item, S::Error>
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
