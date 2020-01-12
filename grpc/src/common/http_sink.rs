use crate::result;
use bytes::Bytes;

pub(crate) trait HttpSink {
    fn state(&self) -> httpbis::SenderState;
    fn send_data(&mut self, data: Bytes) -> result::Result<()>;
}

impl HttpSink for httpbis::ClientRequest {
    fn state(&self) -> httpbis::SenderState {
        self.state()
    }

    fn send_data(&mut self, data: Bytes) -> result::Result<()> {
        self.send_data(data)?;
        Ok(())
    }
}

impl HttpSink for httpbis::ServerResponse {
    fn state(&self) -> httpbis::SenderState {
        self.state()
    }

    fn send_data(&mut self, data: Bytes) -> result::Result<()> {
        self.send_data(data)?;
        Ok(())
    }
}
