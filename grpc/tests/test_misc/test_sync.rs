#![allow(dead_code)]

use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;

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
    pub fn take(&self, wait_for: u32) {
        let mut guard = self.mutex.lock().expect("mutex poisoned");
        loop {
            if *guard == wait_for {
                trace!(
                    "take {} from thread {}",
                    wait_for,
                    thread::current().name().unwrap_or("?")
                );
                *guard += 1;
                self.condvar.notify_all();
                return;
            }

            // otherwise we would stuck here forever
            assert!(
                wait_for > *guard,
                "wait_for = {}, *guard = {}",
                wait_for,
                *guard
            );

            guard = self.condvar.wait(guard).expect("wait");
        }
    }
}
