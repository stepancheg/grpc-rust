use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use futures::task::TaskRc;


#[allow(dead_code)]
pub struct TaskRcMut<A>(TaskRc<RefCell<A>>);

impl<A> Clone for TaskRcMut<A> {
    fn clone(&self) -> Self {
        TaskRcMut(self.0.clone())
    }
}

#[allow(dead_code)]
impl<A> TaskRcMut<A> {

    pub fn new(a: A) -> TaskRcMut<A> {
        TaskRcMut(TaskRc::new(RefCell::new(a)))
    }

    pub fn with<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut A) -> R
    {
        self.0.with(|d| f(&mut *d.borrow_mut()))
    }

}
