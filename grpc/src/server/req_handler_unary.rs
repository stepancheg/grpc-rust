use error;
use httpbis::ServerIncreaseInWindow;
use result;
use server::req_handler::ServerRequestStreamHandler;
use server::req_handler::ServerRequestUnaryHandler;
use std::marker;

pub(crate) struct RequestHandlerUnaryToStream<M, H>
where
    H: ServerRequestUnaryHandler<M>,
    M: 'static,
{
    pub(crate) increase_in_window: ServerIncreaseInWindow,
    pub(crate) handler: H,
    pub(crate) message: Option<M>,
    pub(crate) _marker: marker::PhantomData<M>,
}

impl<M, H> ServerRequestStreamHandler<M> for RequestHandlerUnaryToStream<M, H>
where
    H: ServerRequestUnaryHandler<M>,
    M: 'static,
{
    fn grpc_message(&mut self, message: M, frame_size: u32) -> result::Result<()> {
        self.increase_in_window.data_frame_processed(frame_size);
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

    fn in_window_empty(&mut self, _buffered: usize) -> result::Result<()> {
        if self.increase_in_window.in_window_size() == 0 {
            // TODO: choose something less random
            self.increase_in_window.increase_window(10000)?;
        }
        Ok(())
    }
}
