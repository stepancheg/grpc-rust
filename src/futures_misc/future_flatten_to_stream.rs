use futures::stream::BoxStream;
use futures::stream::Stream;
use futures::Future;
use futures::Poll;


pub fn future_flatten_to_stream<F, T : Send + 'static, E : Send + 'static, S>(f: F) -> BoxStream<T, E>
    where
        S : Stream<Item=T, Error=E> + Send + 'static,
        F : Future<Item=S, Error=E> + Send + 'static,
{
    FutureFlattenToStream {
        inner: FutureFlattenToStreamInner::Future(f)
    }.boxed()
}

enum FutureFlattenToStreamInner<F, S> {
    Future(F),
    Stream(S),
}

pub struct FutureFlattenToStream<F, E> {
    inner: FutureFlattenToStreamInner<F, E>,
}

impl<F, S> Stream for FutureFlattenToStream<F, S>
    where
        S : Stream,
        F : Future<Item=S, Error=S::Error>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let next = match &mut self.inner {
            &mut FutureFlattenToStreamInner::Future(ref mut f) => {
                match f.poll() {
                    Poll::NotReady => return Poll::NotReady,
                    Poll::Err(e) => return Poll::Err(e),
                    Poll::Ok(stream) => stream,
                }
            }
            &mut FutureFlattenToStreamInner::Stream(ref mut s) => {
                return s.poll();
            }
        };

        self.inner = FutureFlattenToStreamInner::Stream(next);
        if let &mut FutureFlattenToStreamInner::Stream(ref mut s) = &mut self.inner {
            s.poll()
        } else {
            unreachable!()
        }
    }
}

