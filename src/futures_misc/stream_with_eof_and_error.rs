use futures::stream::Stream;
use futures::Poll;
use futures::Async;


pub enum ResultOrEof<T, E> {
    Item(T),
    Error(E),
    Eof,
}

impl<T, E> ResultOrEof<T, E> {
    pub fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, op: F) -> ResultOrEof<U, E> {
        match self {
            ResultOrEof::Item(t) => match op(t) {
                Ok(r) => ResultOrEof::Item(r),
                Err(e) => ResultOrEof::Error(e),
            },
            ResultOrEof::Error(e) => ResultOrEof::Error(e),
            ResultOrEof::Eof => ResultOrEof::Eof,
        }
    }
}

impl<T, E> From<Result<T, E>> for ResultOrEof<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(r) => ResultOrEof::Item(r),
            Err(e) => ResultOrEof::Error(e),
        }
    }
}



pub fn stream_with_eof_and_error<T, E, S>(s: S) -> StreamWithEofAndError<S>
    where S : Stream<Item=ResultOrEof<T, E>, Error=E>
{
    StreamWithEofAndError {
        stream: s,
        seen_eof: false,
    }
}

pub struct StreamWithEofAndError<S> {
    stream: S,
    seen_eof: bool,
}

impl<T, E, S> Stream for StreamWithEofAndError<S>
    where S : Stream<Item=ResultOrEof<T, E>, Error=E>
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if self.seen_eof {
                match try_ready!(self.stream.poll()) {
                    None => return Ok(Async::Ready(None)),
                    Some(ResultOrEof::Error(e)) => return Err(e),
                    Some(ResultOrEof::Eof) => panic!("eof after eof"),
                    Some(ResultOrEof::Item(_)) => panic!("item after eof"),
                }
            } else {
                match try_ready!(self.stream.poll()) {
                    None => panic!("expecting explicit eof, got stream eof"),
                    Some(ResultOrEof::Eof) => {
                        self.seen_eof = true;
                        continue;
                    }
                    Some(ResultOrEof::Error(e)) => return Err(e),
                    Some(ResultOrEof::Item(item)) => return Ok(Async::Ready(Some(item))),
                }
            }
        }
    }
}
