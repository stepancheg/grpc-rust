use Metadata;
use tokio_core::reactor::Remote;
use futures::future;
use futures::Poll;
use error;
use futures::Async;
use futures::stream;
use ServerResponseSink;
use ServerResponseUnarySink;

pub struct ServerHandlerContext {
    pub ctx: httpbis::ServerHandlerContext,
    // TODO: move to request
    pub metadata: Metadata,
}

impl ServerHandlerContext {
    pub fn loop_remote(&self) -> Remote {
        self.ctx.loop_remote()
    }

    pub fn spawn_poll_fn<F>(&self, mut f: F)
        where F: FnMut() -> Poll<(), error::Error> + Send + 'static
    {
        self.loop_remote().spawn(move |_handle| {
            future::poll_fn(move || {
                match f() {
                    Ok(r) => Ok(r),
                    Err(e) => {
                        warn!("poll_fn returned error: {:?}", e);
                        Ok(Async::Ready(()))
                    }
                }
            })
        })
    }

    pub fn pump<Resp, S>(&self, mut stream: S, mut dest: ServerResponseSink<Resp>)
        where
            Resp: Send + 'static,
            S: stream::Stream<Item=Resp, Error=error::Error> + Send + 'static
    {
        self.spawn_poll_fn(move || {
            loop {
                if let Async::NotReady = dest.poll()? {
                    return Ok(Async::NotReady);
                }
                match stream.poll()? {
                    Async::NotReady => return Ok(Async::NotReady),
                    Async::Ready(Some(m)) => {
                        dest.send_data(m)?;
                    }
                    Async::Ready(None) => {
                        dest.send_trailers(Metadata::new())?;
                        return Ok(Async::Ready(()));
                    }
                }
            }
        })
    }

    pub fn pump_future<Resp, F>(&self, mut future: F, dest: ServerResponseUnarySink<Resp>)
        where
            Resp: Send + 'static,
            F: future::Future<Item=Resp, Error=error::Error> + Send + 'static
    {
        let mut dest = Some(dest);
        self.spawn_poll_fn(move || {
            loop {
                match future.poll()? {
                    Async::NotReady => return Ok(Async::NotReady),
                    Async::Ready(m) => {
                        dest.take().unwrap().finish(m)?;
                        return Ok(Async::Ready(()));
                    }
                }
            }
        })
    }
}
