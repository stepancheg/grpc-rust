use futures::stream::Stream;
use futures::Future;
use futures;


/// Convert a stream into single element future.
/// It is an error, is stream is empty or has more than one element.
pub fn stream_single<S>(stream: S) -> Box<Future<Item=S::Item, Error=S::Error>>
    where
        S : Stream + 'static,
        S::Item : 'static,
        S::Error : 'static,
{
    Box::new(stream
        .fold(None, |option, item| {
            match option {
                Some(..) => panic!("more than one element"), // TODO: better error
                None => futures::finished::<_, S::Error>(Some(item))
            }
        })
        .map(|option| {
            option.expect("expecting one element, found none") // TODO: better error
        }))
}

pub fn stream_single_send<S>(stream: S) -> Box<Future<Item=S::Item, Error=S::Error> + Send>
    where
        S : Stream + Send + 'static,
        S::Item : Send + 'static,
        S::Error : Send + 'static,
{
    Box::new(stream
        .fold(None, |option, item| {
            match option {
                Some(..) => panic!("more than one element"), // TODO: better error
                None => futures::finished::<_, S::Error>(Some(item))
            }
        })
        .map(|option| {
            option.expect("expecting one element, found none") // TODO: better error
        }))
}
