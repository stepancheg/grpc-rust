use std::cell::RefCell;

use std::sync::Arc;
use std::sync::Mutex;
use std::iter;

use futures::*;
use futures::stream::Stream;
use futures::stream::BoxStream;
use futures::task::TaskData;


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
