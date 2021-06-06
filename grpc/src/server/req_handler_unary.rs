use crate::error;
use crate::result;
use crate::server::req_handler::ServerRequestStreamHandler;
use crate::server::req_handler::ServerRequestUnaryHandler;
use httpbis::IncreaseInWindow;
use std::marker;

pub(crate) struct RequestHandlerUnaryToStream<M, H>
where
    H: ServerRequestUnaryHandler<M>,
    M: Send + 'static,
{
    pub(crate) increase_in_window: IncreaseInWindow,
    pub(crate) handler: H,
    pub(crate) message: Option<M>,
    pub(crate) _marker: marker::PhantomData<M>,
}

impl<M, H> ServerRequestStreamHandler<M> for RequestHandlerUnaryToStream<M, H>
where
    H: ServerRequestUnaryHandler<M>,
    M: Send + 'static,
{
    fn grpc_message(&mut self, message: M, frame_size: usize) -> result::Result<()> {
        self.increase_in_window.data_frame_processed(frame_size);
        self.increase_in_window.increase_window_auto()?;
        if let Some(_) = self.message {
            return Err(error::Error::Other("more than one message in a stream"));
        }
        self.message = Some(message);
        Ok(())
    }

    fn end_stream(&mut self) -> result::Result<()> {
        match self.message.take() {
            Some(message) => self.handler.grpc_message(message),
            None => Err(error::Error::Other("no message, end of stream")),
        }
    }

    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()> {
        // TODO: overflow check
        self.increase_in_window
            .increase_window_auto_above(buffered as u32)?;
        Ok(())
    }
}
