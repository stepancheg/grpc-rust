use futures::stream::Stream;
use futures::Poll;


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
                match self.stream.poll() {
                    Poll::NotReady => return Poll::NotReady,
                    Poll::Ok(None) => return Poll::Ok(None),
                    Poll::Ok(Some(_)) => panic!("item after eof"),
                    Poll::Err(e) => return Poll::Err(e),
                }
            } else {
                match self.stream.poll() {
                    Poll::NotReady => return Poll::NotReady,
                    Poll::Ok(None) => panic!("expecting explicit eof"),
                    Poll::Ok(Some(StreamWithEofMessage::Eof)) => {
                        self.seen_eof = true;
                        continue;
                    }
                    Poll::Ok(Some(StreamWithEofMessage::Item(item))) => return Poll::Ok(Some(item)),
                    Poll::Err(e) => return Poll::Err(e),
                }
            }
        }
    }
}
