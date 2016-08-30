use futures::stream::Stream;
use futures::Poll;


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



pub fn stream_with_eof_and_error<S>(s: S) -> StreamWithEofAndError<S> {
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
                match self.stream.poll() {
                    Poll::NotReady => return Poll::NotReady,
                    Poll::Ok(None) => return Poll::Ok(None),
                    Poll::Ok(Some(ResultOrEof::Error(e))) => return Poll::Err(e),
                    Poll::Ok(Some(ResultOrEof::Eof)) => panic!("eof after eof"),
                    Poll::Ok(Some(ResultOrEof::Item(_))) => panic!("item after eof"),
                    Poll::Err(e) => return Poll::Err(e),
                }
            } else {
                match self.stream.poll() {
                    Poll::NotReady => return Poll::NotReady,
                    Poll::Ok(None) => panic!("expecting explicit eof"),
                    Poll::Ok(Some(ResultOrEof::Eof)) => {
                        self.seen_eof = true;
                        continue;
                    }
                    Poll::Ok(Some(ResultOrEof::Error(e))) => return Poll::Err(e),
                    Poll::Ok(Some(ResultOrEof::Item(item))) => return Poll::Ok(Some(item)),
                    Poll::Err(e) => return Poll::Err(e),
                }
            }
        }
    }
}
