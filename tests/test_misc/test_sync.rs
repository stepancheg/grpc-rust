use std::sync::Mutex;
use std::sync::Condvar;


/// Super duper utility to test multithreaded apps.
pub struct TestSync {
    mutex: Mutex<u32>,
    condvar: Condvar,
}

impl TestSync {
    pub fn new() -> TestSync {
        TestSync {
            mutex: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    /// Wait for requested value to appear
    /// and increment contained value.
    pub fn take(&self, value: u32) {
        let mut guard = self.mutex.lock().unwrap();
        loop {
            if *guard == value {
                //println!("take {} from thread {}", value, thread::current().name().unwrap_or("?"));
                *guard += 1;
                self.condvar.notify_all();
                return;
            }

            // otherwise we would stuck here forever
            assert!(value > *guard);

            guard = self.condvar.wait(guard).unwrap();
        }
    }
}
