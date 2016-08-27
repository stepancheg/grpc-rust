use std::cell::RefCell;

use std::sync::Arc;
use std::sync::Mutex;
use std::iter;

use futures::*;
use futures::stream::Stream;
use futures::stream::BoxStream;
use futures::task::TaskData;


pub struct FutureToStream<F> {
    future: Option<F>,
}

pub fn future_to_stream<F : Future>(f: F) -> FutureToStream<F> {
    FutureToStream {
        future: Some(f)
    }
}

impl<F : Future> Stream for FutureToStream<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let r = match &mut self.future {
            &mut None => return Poll::Ok(None),
            &mut Some(ref mut future) => match future.poll() {
                Poll::NotReady => return Poll::NotReady,
                Poll::Err(e) => Poll::Err(e),
                Poll::Ok(r) => Poll::Ok(Some(r)),
            }
        };

        self.future.take();

        r
    }
}


#[allow(dead_code)]
pub fn future_success<T : Send + 'static, E : Send + 'static>(t: T) -> BoxFuture<T, E> {
    done(Ok(t))
        .boxed()
}


#[allow(dead_code)]
pub fn stream_repeat<T : Clone + Send + 'static, E>(t: T) -> BoxStream<T, E> {
    let ts = iter::repeat(t).map(|t| Ok(t));
    stream::iter(ts)
        .boxed()
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
