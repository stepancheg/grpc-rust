use futures::Future;
use futures::Poll;
use futures::Async;
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
                return Ok(Async::Ready(None))
            },
            &mut FutureToStreamState::Done => {
                panic!("cannot poll after eof");
            },
            &mut FutureToStreamState::Future(ref mut future) => try_ready!(future.poll()),
        };

        self.future = FutureToStreamState::Eof;

        Ok(Async::Ready(Some(r)))
    }
}
