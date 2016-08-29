use std::cell::RefCell;

use std::sync::Arc;
use std::sync::Mutex;
use std::iter;

use futures;
use futures::*;
use futures::stream::Stream;
use futures::stream::BoxStream;
use futures::task::TaskData;


enum FutureToStreamState<F> {
    Future(F),
    Eof,
    Done,
}

pub struct FutureToStream<F> {
    future: FutureToStreamState<F>,
}

pub fn future_to_stream<F : Future>(f: F) -> FutureToStream<F> {
    FutureToStream {
        future: FutureToStreamState::Future(f)
    }
}

impl<F : Future> Stream for FutureToStream<F> {
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


pub fn single_element<S>(stream: S) -> BoxFuture<S::Item, S::Error>
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


pub fn err_stream<T : Send + 'static, E : Send + 'static>(error: E) -> BoxStream<T, E> {
    future_to_stream(failed(error))
        .boxed()
}


#[allow(dead_code)]
pub fn stream_repeat<T : Clone + Send + 'static, E>(t: T) -> BoxStream<T, E> {
    let ts = iter::repeat(t).map(|t| Ok(t));
    stream::iter(ts)
        .boxed()
}


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


//#[derive(Clone)]
#[allow(dead_code)]
pub struct TaskDataMut<A>(TaskData<RefCell<A>>);

impl<A> Clone for TaskDataMut<A> {
    fn clone(&self) -> Self {
        TaskDataMut(self.0.clone())
    }
}

#[allow(dead_code)]
impl<A> TaskDataMut<A> {

    pub fn new(a: A) -> TaskDataMut<A> {
        TaskDataMut(TaskData::new(RefCell::new(a)))
    }

    pub fn with<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut A) -> R
    {
        self.0.with(|d| f(&mut *d.borrow_mut()))
    }

}

pub struct TaskDataMutex<A>(TaskData<Arc<Mutex<A>>>);

impl<A> Clone for TaskDataMutex<A> {
    fn clone(&self) -> Self {
        TaskDataMutex(self.0.clone())
    }
}

impl<A> TaskDataMutex<A> {

    pub fn new(a: A) -> TaskDataMutex<A> {
        TaskDataMutex(TaskData::new(Arc::new(Mutex::new(a))))
    }

    pub fn with<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut A) -> R
    {
        self.0.with(|d| f(&mut *d.lock().unwrap()))
    }

}
