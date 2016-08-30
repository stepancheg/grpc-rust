use futures::Future;
use futures::Poll;
use futures::stream::Stream;


enum FutureToStreamState<F> {
    Future(F),
    Eof,
    Done,
}

pub struct FutureToStreamOnce<F> {
    future: FutureToStreamState<F>,
}

pub fn future_to_stream_once<F : Future>(f: F) -> FutureToStreamOnce<F> {
    FutureToStreamOnce {
        future: FutureToStreamState::Future(f)
    }
}

impl<F : Future> Stream for FutureToStreamOnce<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let r = match &mut self.future {
            r @ &mut FutureToStreamState::Eof => {
                *r = FutureToStreamState::Done;
                return Poll::Ok(None)
            },
            &mut FutureToStreamState::Done => {
                panic!("cannot poll after eof");
            },
            &mut FutureToStreamState::Future(ref mut future) => match future.poll() {
                Poll::NotReady => return Poll::NotReady,
                Poll::Err(e) => return Poll::Err(e),
                Poll::Ok(r) => r,
            }
        };

        self.future = FutureToStreamState::Eof;

        Poll::Ok(Some(r))
    }
}
