use crate::client::types::ClientTypes;
use crate::common::sink::SinkCommonUntyped;
use crate::common::sink::SinkUntyped;
use crate::common::sink::{MessageToBeSerialized, SinkCommon};

use crate::result;
use futures::task::Context;
use httpbis;

use futures::future;
use std::future::Future;
use std::task::Poll;

pub struct ClientRequestSinkUntyped {
    pub(crate) common: SinkCommonUntyped<ClientTypes>,
}

impl SinkUntyped for ClientRequestSinkUntyped {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>> {
        self.common.http.poll(cx)
    }

    fn send_message(&mut self, message: &dyn MessageToBeSerialized) -> result::Result<()> {
        self.common.send_message(message)?;
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
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), httpbis::StreamDead>> {
        self.common.poll(cx)
    }

    pub fn wait<'a>(&'a mut self) -> impl Future<Output = Result<(), httpbis::StreamDead>> + 'a {
        future::poll_fn(move |cx| self.poll(cx))
    }

    pub fn send_data(&mut self, message: Req) -> result::Result<()> {
        self.common.send_data(message)
    }

    pub fn finish(&mut self) -> result::Result<()> {
        self.common.sink.finish()
    }
}
