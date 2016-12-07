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



pub fn stream_with_eof_and_error<T, E, S, F>(s: S, missed_eof: F) -> StreamWithEofAndError<S, F>
    where
        S : Stream<Item=ResultOrEof<T, E>, Error=E>,
        F : FnOnce() -> E,
{
    StreamWithEofAndError {
        stream: s,
        seen_eof: false,
        missed_eof: Some(missed_eof),
    }
}

pub struct StreamWithEofAndError<S, F> {
    stream: S,
    seen_eof: bool,
    missed_eof: Option<F>,
}

impl<T, E, S, F> Stream for StreamWithEofAndError<S, F>
    where
        S : Stream<Item=ResultOrEof<T, E>, Error=E>,
        F : FnOnce() -> E,
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
                    None => {
                        match self.missed_eof.take() {
                            Some(f) => return Err(f()),
                            None => {
                                warn!("poll after eof seen but missed_eof not set, setting self.seen_eof = true");
                                self.seen_eof = true;
                            }
                        }
                    }
                    Some(ResultOrEof::Eof) => {
                        self.seen_eof = true;
                        return Ok(Async::Ready(None));
                    }
                    Some(ResultOrEof::Error(e)) => return Err(e),
                    Some(ResultOrEof::Item(item)) => return Ok(Async::Ready(Some(item))),
                }
            }
        }
    }
}
