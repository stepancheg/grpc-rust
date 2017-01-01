use futures::stream::Stream;
use futures::Poll;
use futures::Async;


#[allow(dead_code)]
pub enum StreamWithEofMessage<T> {
    Item(T),
    Eof,
}

#[allow(dead_code)]
pub fn stream_with_eof<S>(s: S) -> StreamWithEof<S> {
    StreamWithEof {
        stream: s,
        seen_eof: false,
    }
}

#[allow(dead_code)]
pub struct StreamWithEof<S> {
    stream: S,
    seen_eof: bool,
}

impl<T, E, S> Stream for StreamWithEof<S>
    where S : Stream<Item=StreamWithEofMessage<T>, Error=E>
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if self.seen_eof {
                match try_ready!(self.stream.poll()) {
                    None => return Ok(Async::Ready(None)),
                    Some(_) => panic!("item after eof"),
                }
            } else {
                match try_ready!(self.stream.poll()) {
                    None => panic!("expecting explicit eof"),
                    Some(StreamWithEofMessage::Eof) => {
                        self.seen_eof = true;
                        continue;
                    }
                    Some(StreamWithEofMessage::Item(item)) => return Ok(Async::Ready(Some(item))),
                }
            }
        }
    }
}
