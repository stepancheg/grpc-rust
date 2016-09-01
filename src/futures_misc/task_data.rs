use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use futures::task::TaskRc;


//#[derive(Clone)]
#[allow(dead_code)]
pub struct TaskDataMut<A>(TaskRc<RefCell<A>>);

impl<A> Clone for TaskDataMut<A> {
    fn clone(&self) -> Self {
        TaskDataMut(self.0.clone())
    }
}

#[allow(dead_code)]
impl<A> TaskDataMut<A> {

    pub fn new(a: A) -> TaskDataMut<A> {
        TaskDataMut(TaskRc::new(RefCell::new(a)))
    }

    pub fn with<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut A) -> R
    {
        self.0.with(|d| f(&mut *d.borrow_mut()))
    }

}

pub struct TaskDataMutex<A>(TaskRc<Arc<Mutex<A>>>);

impl<A> Clone for TaskDataMutex<A> {
    fn clone(&self) -> Self {
        TaskDataMutex(self.0.clone())
    }
}

impl<A> TaskDataMutex<A> {

    pub fn new(a: A) -> TaskDataMutex<A> {
        TaskDataMutex(TaskRc::new(Arc::new(Mutex::new(a))))
    }

    pub fn with<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut A) -> R
    {
        self.0.with(|d| f(&mut *d.lock().unwrap()))
    }

}
