use futures::future;
use futures::StreamExt;

use crate::Metadata;

use futures::stream;

use crate::ServerResponseSink;
use crate::ServerResponseUnarySink;
use std::future::Future;
use std::task::Context;
use std::task::Poll;
use tokio::runtime::Handle;

/// An object passed to server handlers.
pub struct ServerHandlerContext {
    pub ctx: httpbis::ServerHandlerContext,
    // TODO: move to request
    pub metadata: Metadata,
}

impl ServerHandlerContext {
    /// Tokio event loop handle (can be used to spawn a future for example).
    pub fn loop_remote(&self) -> Handle {
        self.ctx.loop_remote()
    }

    /// Spawn a future, ignore result.
    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Output = crate::Result<()>> + Send + 'static,
    {
        self.loop_remote().spawn(async {
            if let Err(e) = f.await {
                warn!("spaned future returned error: {:?}", e);
            }
        });
    }

    /// Spawn a poll_fn future. Function error is ignored.
    pub fn spawn_poll_fn<F>(&self, f: F)
    where
        F: FnMut(&mut Context<'_>) -> Poll<crate::Result<()>> + Send + 'static,
    {
        self.spawn(future::poll_fn(f))
    }

    /// Spawn a future which polls stream and sends items to [`ServerResponseSink`].
    pub fn pump<Resp, S>(&self, mut stream: S, mut dest: ServerResponseSink<Resp>)
    where
        Resp: Send + 'static,
        S: stream::Stream<Item = crate::Result<Resp>> + Unpin + Send + 'static,
    {
        self.spawn(async move {
            while let Some(m) = stream.next().await {
                dest.send_data(m?)?;
            }
            dest.send_trailers(Metadata::new())
        });
    }

    /// Spawn the provided future in tokio even loop, send result to given sink.
    pub fn pump_future<Resp, F>(&self, future: F, dest: ServerResponseUnarySink<Resp>)
    where
        Resp: Send + 'static,
        F: Future<Output = crate::Result<Resp>> + Unpin + Send + 'static,
    {
        self.spawn(async move {
            let m = future.await?;
            dest.finish(m)
        });
    }
}
