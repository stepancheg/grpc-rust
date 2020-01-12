use bytes::Bytes;
use error;
use futures::sync::mpsc;
use httpbis::Headers;
use httpbis::ServerIncreaseInWindow;
use marshall::Marshaller;
use or_static::arc::ArcOrStatic;
use proto::grpc_frame::parse_grpc_frame_from_bytes;
use result;
use server::req_handler_unary::RequestHandlerUnaryToStream;
use server::req_stream::ServerRequestStreamSenderHandler;
use std::marker;
use Metadata;
use ServerRequestStream;

pub(crate) trait ServerRequestStreamHandlerUntyped: 'static {
    fn grpc_message(&mut self, message: Bytes, frame_size: u32) -> result::Result<()>;
    fn end_stream(&mut self) -> result::Result<()>;
    fn error(&mut self, error: error::Error) -> result::Result<()>;
    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()>;
}

pub trait ServerRequestStreamHandler<M>: 'static {
    fn grpc_message(&mut self, message: M, frame_size: u32) -> result::Result<()>;
    fn end_stream(&mut self) -> result::Result<()>;
    fn error(&mut self, error: error::Error) -> result::Result<()> {
        Err(error)
    }
    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()>;
}

pub trait ServerRequestUnaryHandler<M>: 'static {
    fn grpc_message(&mut self, message: M) -> result::Result<()>;
    fn error(&mut self, error: error::Error) -> result::Result<()> {
        Err(error)
    }
}

struct ServerStreamStreamHandlerUntypedHandler<H: ServerRequestStreamHandlerUntyped> {
    buf: Bytes,
    handler: H,
}

impl<H: ServerRequestStreamHandlerUntyped> ServerStreamStreamHandlerUntypedHandler<H> {
    fn process_buf(&mut self) -> result::Result<()> {
        loop {
            let old_len = self.buf.len();
            let grpc_message = match parse_grpc_frame_from_bytes(&mut self.buf)? {
                Some(grpc_message) => grpc_message,
                None => return Ok(()),
            };
            let consumed = old_len - self.buf.len();

            // TODO: checked cast
            self.handler.grpc_message(grpc_message, consumed as u32)?;
        }
    }

    fn end_stream(&mut self) -> result::Result<()> {
        if !self.buf.is_empty() {
            return Err(error::Error::Other("not complete frames").into());
        }

        self.handler.end_stream()?;
        Ok(())
    }
}

impl<H: ServerRequestStreamHandlerUntyped> httpbis::ServerStreamHandler
    for ServerStreamStreamHandlerUntypedHandler<H>
{
    fn data_frame(&mut self, data: Bytes, end_stream: bool) -> httpbis::Result<()> {
        if self.buf.is_empty() {
            self.buf = data;
        } else {
            self.buf.extend_from_slice(&data);
        }

        self.process_buf()?;

        if end_stream {
            self.end_stream()?;
            return Ok(());
        }

        if self.buf.len() != 0 {
            trace!(
                "buffer processed, still contains {}, calling handler",
                self.buf.len()
            );
            self.handler.buffer_processed(self.buf.len())?;
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
    marshaller: ArcOrStatic<Marshaller<M>>,
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

    fn error(&mut self, error: error::Error) -> result::Result<()> {
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
                    buf: Bytes::new(),
                    handler,
                },
                r,
            )
        })
    }
}

pub struct ServerRequest<'a, M: 'static> {
    pub(crate) req: ServerRequestUntyped<'a>,
    pub(crate) marshaller: ArcOrStatic<Marshaller<M>>,
}

impl<'a, M: Send + 'static> ServerRequest<'a, M> {
    pub fn metadata(&self) -> Metadata {
        // TODO: take, not clone
        // TODO: do not unwrap
        Metadata::from_headers(self.req.req.headers.clone()).unwrap()
    }

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
