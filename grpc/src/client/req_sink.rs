use bytes::Bytes;
use client::types::ClientTypes;
use common::sink::SinkCommon;
use common::sink::SinkCommonUntyped;
use common::sink::SinkUntyped;
use futures::future;
use futures::future::Future;
use futures::Poll;
use httpbis;
use httpbis::StreamDead;
use result;

pub struct ClientRequestSinkUntyped {
    pub(crate) common: SinkCommonUntyped<ClientTypes>,
}

impl SinkUntyped for ClientRequestSinkUntyped {
    fn poll(&mut self) -> Poll<(), httpbis::StreamDead> {
        self.common.http.poll()
    }

    fn send_data(&mut self, message: Bytes) -> result::Result<()> {
        self.common.send_data(message)?;
        Ok(())
    }
}

impl ClientRequestSinkUntyped {
    pub fn finish(&mut self) -> result::Result<()> {
        self.common.http.close()?;
        Ok(())
    }
}

pub struct ClientRequestSink<Req: Send + 'static> {
    pub(crate) common: SinkCommon<Req, ClientTypes>,
}

impl<Req: Send> ClientRequestSink<Req> {
    pub fn poll(&mut self) -> Poll<(), httpbis::StreamDead> {
        self.common.poll()
    }

    pub fn block_wait(&mut self) -> Result<(), StreamDead> {
        future::poll_fn(|| self.poll()).wait()
    }

    pub fn send_data(&mut self, message: Req) -> result::Result<()> {
        self.common.send_data(message)
    }

    pub fn finish(&mut self) -> result::Result<()> {
        self.common.sink.finish()
    }
}
