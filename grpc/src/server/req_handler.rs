use bytes::Bytes;

use crate::marshall::Marshaller;
use crate::or_static::arc::ArcOrStatic;
use crate::result;
use crate::server::req_handler_unary::RequestHandlerUnaryToStream;
use crate::server::req_stream::ServerRequestStreamSenderHandler;
use httpbis::Headers;
use httpbis::ServerIncreaseInWindow;

use std::marker;

use futures::channel::mpsc;

use crate::proto::grpc_frame_parser::GrpcFrameParser;
use crate::Metadata;
use crate::ServerRequestStream;

pub(crate) trait ServerRequestStreamHandlerUntyped: Send + 'static {
    fn grpc_message(&mut self, message: Bytes, frame_size: u32) -> result::Result<()>;
    fn end_stream(&mut self) -> result::Result<()>;
    fn error(&mut self, error: crate::Error) -> result::Result<()>;
    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()>;
}

pub trait ServerRequestStreamHandler<M>: Send + 'static {
    fn grpc_message(&mut self, message: M, frame_size: u32) -> result::Result<()>;
    fn end_stream(&mut self) -> result::Result<()>;
    fn error(&mut self, error: crate::Error) -> result::Result<()> {
        Err(error)
    }
    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()>;
}

pub trait ServerRequestUnaryHandler<M>: Send + 'static
where
    M: Send,
{
    fn grpc_message(&mut self, message: M) -> result::Result<()>;
    fn error(&mut self, error: crate::Error) -> result::Result<()> {
        Err(error)
    }
}

struct ServerStreamStreamHandlerUntypedHandler<H: ServerRequestStreamHandlerUntyped> {
    buf: GrpcFrameParser,
    handler: H,
}

impl<H: ServerRequestStreamHandlerUntyped> ServerStreamStreamHandlerUntypedHandler<H> {
    fn process_buf(&mut self) -> result::Result<()> {
        let mut total_consumed = 0;
        while let Some((grpc_message, consumed)) = self.buf.next_frame()? {
            // TODO: checked cast
            self.handler.grpc_message(grpc_message, consumed as u32)?;
            total_consumed += consumed;
        }

        if total_consumed != 0 {
            self.handler.buffer_processed(total_consumed)?;
        }

        Ok(())
    }

    fn end_stream(&mut self) -> result::Result<()> {
        if !self.buf.is_empty() {
            return Err(crate::Error::Other("not complete frames").into());
        }

        self.handler.end_stream()?;
        Ok(())
    }
}

impl<H: ServerRequestStreamHandlerUntyped> httpbis::ServerRequestStreamHandler
    for ServerStreamStreamHandlerUntypedHandler<H>
{
    fn data_frame(&mut self, data: Bytes, end_stream: bool) -> httpbis::Result<()> {
        self.buf.enqueue(data);

        self.process_buf()?;

        if end_stream {
            self.buf.check_empty()?;

            self.end_stream()?;
            return Ok(());
        }

        Ok(())
    }

    fn trailers(&mut self, trailers: Headers) -> httpbis::Result<()> {
        // there are no trailers in gRPC request
        drop(trailers);

        // trigger error if buf is not empty
        self.process_buf()?;

        self.handler.end_stream()?;

        Ok(())
    }

    fn error(&mut self, error: httpbis::Error) -> httpbis::Result<()> {
        self.handler.error(error.into())?;
        Ok(())
    }
}

struct ServerRequestStreamHandlerHandler<M: 'static, H: ServerRequestStreamHandler<M>> {
    handler: H,
    marshaller: ArcOrStatic<dyn Marshaller<M>>,
}

impl<M, H: ServerRequestStreamHandler<M>> ServerRequestStreamHandlerUntyped
    for ServerRequestStreamHandlerHandler<M, H>
{
    fn grpc_message(&mut self, message: Bytes, frame_size: u32) -> result::Result<()> {
        let message = self.marshaller.read(message)?;
        self.handler.grpc_message(message, frame_size)
    }

    fn end_stream(&mut self) -> result::Result<()> {
        self.handler.end_stream()
    }

    fn error(&mut self, error: crate::Error) -> result::Result<()> {
        self.handler.error(error)
    }

    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()> {
        self.handler.buffer_processed(buffered)
    }
}

pub(crate) struct ServerRequestUntyped<'a> {
    pub(crate) req: httpbis::ServerRequest<'a>,
}

impl<'a> ServerRequestUntyped<'a> {
    pub fn register_stream_handler<F, H, R>(self, handler: F) -> R
    where
        H: ServerRequestStreamHandlerUntyped,
        F: FnOnce(ServerIncreaseInWindow) -> (H, R),
    {
        self.req.register_stream_handler(|increase_in_window| {
            let (handler, r) = handler(increase_in_window);
            (
                ServerStreamStreamHandlerUntypedHandler {
                    buf: GrpcFrameParser::default(),
                    handler,
                },
                r,
            )
        })
    }
}

/// Streaming server request.
///
/// This object is passed to server handlers.
pub struct ServerRequest<'a, M: 'static> {
    pub(crate) req: ServerRequestUntyped<'a>,
    pub(crate) marshaller: ArcOrStatic<dyn Marshaller<M>>,
}

impl<'a, M: Send + 'static> ServerRequest<'a, M> {
    /// Get request metadata.
    pub fn metadata(&self) -> Metadata {
        // TODO: take, not clone
        // TODO: do not unwrap
        Metadata::from_headers(self.req.req.headers.clone()).unwrap()
    }

    /// Register server stream handler.
    ///
    /// This is low level operation, hard to use. Most users use
    /// [`into_stream`](ServerRequest::into_stream) operation.
    pub fn register_stream_handler<F, H, R>(self, handler: F) -> R
    where
        H: ServerRequestStreamHandler<M>,
        F: FnOnce(ServerIncreaseInWindow) -> (H, R),
    {
        let marshaller = self.marshaller.clone();
        self.req.register_stream_handler(|increase_in_window| {
            let (handler, r) = handler(increase_in_window);
            (
                ServerRequestStreamHandlerHandler {
                    handler,
                    marshaller,
                },
                r,
            )
        })
    }

    /// Register basic stream handler (without manual control flow).
    ///
    /// This is low level operation, hard to use. Most users use
    /// [`into_stream`](ServerRequest::into_stream) operation.
    pub fn register_stream_handler_basic<H>(self, handler: H)
    where
        H: FnMut(Option<M>) -> result::Result<()> + Send + 'static,
    {
        self.register_stream_handler(move |increase_in_window| {
            struct Handler<M, F>
            where
                M: Send + 'static,
                F: FnMut(Option<M>) -> result::Result<()> + Send + 'static,
            {
                increase_in_window: ServerIncreaseInWindow,
                handler: F,
                _marker: marker::PhantomData<M>,
            }

            impl<M, F> ServerRequestStreamHandler<M> for Handler<M, F>
            where
                M: Send + 'static,
                F: FnMut(Option<M>) -> result::Result<()> + Send + 'static,
            {
                fn grpc_message(&mut self, message: M, frame_size: u32) -> result::Result<()> {
                    (self.handler)(Some(message))?;
                    self.increase_in_window.data_frame_processed(frame_size);
                    self.increase_in_window.increase_window_auto()?;
                    Ok(())
                }

                fn end_stream(&mut self) -> result::Result<()> {
                    (self.handler)(None)
                }

                fn buffer_processed(&mut self, buffered: usize) -> result::Result<()> {
                    // TODO: overflow check
                    self.increase_in_window
                        .increase_window_auto_above(buffered as u32)?;
                    Ok(())
                }
            }

            (
                Handler {
                    increase_in_window,
                    handler,
                    _marker: marker::PhantomData,
                },
                (),
            )
        })
    }

    /// Register unary request handler.
    ///
    /// Few people need this operation.
    pub fn register_unary_handler<H>(self, handler: H)
    where
        H: ServerRequestUnaryHandler<M>,
    {
        // TODO: max message size
        self.register_stream_handler(|increase_in_window| {
            (
                RequestHandlerUnaryToStream {
                    increase_in_window,
                    handler,
                    message: None,
                    _marker: marker::PhantomData,
                },
                (),
            )
        })
    }

    /// Convert request into easy to use [`ServerRequestStream`] object.
    pub fn into_stream(self) -> ServerRequestStream<M> {
        self.register_stream_handler(|increase_in_window| {
            let (tx, rx) = mpsc::unbounded();
            (
                ServerRequestStreamSenderHandler { sender: tx },
                ServerRequestStream {
                    req: rx,
                    increase_in_window,
                },
            )
        })
    }
}
