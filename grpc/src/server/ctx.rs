use futures::{future, FutureExt, StreamExt};

use crate::Metadata;

use futures::stream;

use crate::ServerResponseSink;
use crate::ServerResponseUnarySink;
use std::future::Future;
use std::task::Context;
use std::task::Poll;
use tokio::runtime::Handle;

pub struct ServerHandlerContext {
    pub ctx: httpbis::ServerHandlerContext,
    // TODO: move to request
    pub metadata: Metadata,
}

impl ServerHandlerContext {
    pub fn loop_remote(&self) -> Handle {
        self.ctx.loop_remote()
    }

    pub fn spawn_poll_fn<F>(&self, mut f: F)
    where
        F: FnMut(&mut Context<'_>) -> Poll<crate::Result<()>> + Send + 'static,
    {
        self.loop_remote()
            .spawn(future::poll_fn(move |cx| match f(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Ok(())) => Poll::Ready(()),
                Poll::Ready(Err(e)) => {
                    warn!("poll_fn returned error: {:?}", e);
                    Poll::Ready(())
                }
            }));
    }

    pub fn pump<Resp, S>(&self, mut stream: S, mut dest: ServerResponseSink<Resp>)
    where
        Resp: Send + 'static,
        S: stream::Stream<Item = crate::Result<Resp>> + Unpin + Send + 'static,
    {
        self.spawn_poll_fn(move |cx| loop {
            if let Poll::Pending = dest.poll(cx)? {
                return Poll::Pending;
            }
            match stream.poll_next_unpin(cx)? {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(m)) => {
                    dest.send_data(m)?;
                }
                Poll::Ready(None) => {
                    dest.send_trailers(Metadata::new())?;
                    return Poll::Ready(Ok(()));
                }
            }
        })
    }

    pub fn pump_future<Resp, F>(&self, mut future: F, dest: ServerResponseUnarySink<Resp>)
    where
        Resp: Send + 'static,
        F: Future<Output = crate::Result<Resp>> + Unpin + Send + 'static,
    {
        let mut dest = Some(dest);
        self.spawn_poll_fn(move |cx| loop {
            match future.poll_unpin(cx)? {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(m) => {
                    dest.take().unwrap().finish(m)?;
                    return Poll::Ready(Ok(()));
                }
            }
        })
    }
}
