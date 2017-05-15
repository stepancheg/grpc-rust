# grpc-rust FAQ

## Q: I get an error when I try to make mutable service

Parameters in service are immutable intentionally because operations can be executed concurrently, so `self` should be protected against access from multiple threads. For example, mutable value can be wrapped in `Arc<Mutex<T>>`.

```rust
...

use std::sync::{Arc, Mutex};

struct CounterServiceImpl {
    requests: Arc<Mutex<u64>>,
}

impl CounterServiceImpl {
    fn new() -> CounterServiceImpl {
        CounterServiceImpl { requests: Arc::new(Mutex::new(0)) }
    }
}

impl CounterService for CounterServiceImpl {
    fn Count(&self, req: Request) -> grpc::Result<Reply> {
        println!("Request url: {}", req.get_url());
        *self.requests.lock().unwrap() += 1;
        Ok(Reply::new())
    }
}
```
