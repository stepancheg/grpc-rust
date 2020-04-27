pub(crate) mod http_request_to_grpc_frames_typed;
pub(crate) mod http_response_to_grpc_frames;
pub(crate) mod http_response_to_grpc_frames_typed;
pub(crate) mod req_sink;
pub(crate) mod types;

use bytes::Bytes;

use httpbis::ClientTlsOption;
use httpbis::Header;
use httpbis::Headers;
use httpbis::HttpScheme;

use crate::method::MethodDescriptor;

use crate::result;

use crate::client::http_request_to_grpc_frames_typed::http_req_to_grpc_frames_typed;
use crate::client::http_response_to_grpc_frames_typed::http_response_to_grpc_frames_typed;
use crate::client::req_sink::ClientRequestSink;
use crate::error;
use futures::future;

use futures::future::TryFutureExt;

use crate::or_static::arc::ArcOrStatic;
use crate::proto::grpc_frame::write_grpc_frame_cb;
use crate::req::*;
use crate::resp::*;
use std::future::Future;
use std::pin::Pin;
use tokio::runtime::Handle;

/// Client configuration.
#[derive(Default, Debug, Clone)]
pub struct ClientConf {
    /// HTTP/2 client configuration.
    pub http: httpbis::ClientConf,
}

impl ClientConf {
    /// Create default configuration.
    pub fn new() -> ClientConf {
        Default::default()
    }
}

enum ClientBuilderType<'a> {
    Tcp { port: u16, host: &'a str },
    Unix { socket: &'a str },
}

enum Tls<T: tls_api::TlsConnector> {
    Explict(ClientTlsOption<T>),
    Implicit,
    None,
}

/// Builder for [`Client`].
pub struct ClientBuilder<'a, T: tls_api::TlsConnector> {
    client_type: ClientBuilderType<'a>,
    http_scheme: HttpScheme,
    event_loop: Option<Handle>,
    pub conf: ClientConf,
    tls: Tls<T>,
}

impl<'a, T: tls_api::TlsConnector> ClientBuilder<'a, T> {
    pub fn with_event_loop(mut self, event_loop: Handle) -> Self {
        self.event_loop = Some(event_loop);
        self
    }

    pub fn conf(mut self, config: ClientConf) -> Self {
        self.conf = config;
        self
    }

    pub fn build(self) -> result::Result<Client> {
        let mut builder = httpbis::ClientBuilder::<T>::new();
        let mut conf = self.conf;
        conf.http.thread_name = Some(
            conf.http
                .thread_name
                .unwrap_or_else(|| "grpc-client-loop".to_owned()),
        );
        let (host, port) = match self.client_type {
            ClientBuilderType::Tcp { host, port } => {
                if self.http_scheme == HttpScheme::Https {
                    builder.set_tls(host)?;
                }
                builder.set_addr((host, port))?;
                (host, Some(port))
            }
            ClientBuilderType::Unix { socket } => {
                builder.set_unix_addr(socket)?;
                (socket, None)
            }
        };
        builder.event_loop = self.event_loop;
        builder.conf = conf.http;
        match self.tls {
            Tls::Explict(tls) => {
                builder.tls = tls;
            }
            Tls::Implicit | Tls::None => {}
        }

        Ok(Client {
            client: ::std::sync::Arc::new(builder.build()?),
            host: host.to_owned(),
            http_scheme: self.http_scheme,
            port,
        })
    }
}

impl<'a> ClientBuilder<'a, tls_api_stub::TlsConnector> {
    pub fn new(host: &'a str, port: u16) -> Self {
        ClientBuilder {
            client_type: ClientBuilderType::Tcp { port, host },
            http_scheme: HttpScheme::Http,
            event_loop: None,
            conf: Default::default(),
            tls: Tls::None,
        }
    }

    pub fn new_unix(addr: &'a str) -> Self {
        ClientBuilder {
            client_type: ClientBuilderType::Unix { socket: addr },
            http_scheme: HttpScheme::Http,
            event_loop: None,
            conf: Default::default(),
            tls: Tls::None,
        }
    }

    pub fn tls<TLS: tls_api::TlsConnector>(self) -> ClientBuilder<'a, TLS> {
        ClientBuilder {
            client_type: self.client_type,
            http_scheme: HttpScheme::Https,
            event_loop: self.event_loop,
            conf: self.conf,
            tls: Tls::Implicit,
        }
    }

    pub fn explicit_tls<TLS: tls_api::TlsConnector>(
        self,
        tls: ClientTlsOption<TLS>,
    ) -> ClientBuilder<'a, TLS> {
        ClientBuilder {
            client_type: self.client_type,
            http_scheme: tls.http_scheme(),
            event_loop: self.event_loop,
            conf: self.conf,
            tls: Tls::Explict(tls),
        }
    }
}

/// gRPC client implementation.
/// Used by generated code.
#[derive(Debug)]
pub struct Client {
    client: ::std::sync::Arc<httpbis::Client>,
    host: String,
    http_scheme: HttpScheme,
    port: Option<u16>,
}

impl Client {
    fn call_impl<Req, Resp>(
        &self,
        options: RequestOptions,
        req: Option<Req>,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> Pin<
        Box<
            dyn Future<Output = crate::Result<(ClientRequestSink<Req>, StreamingResponse<Resp>)>>
                + Send,
        >,
    >
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        let mut authority = self.host.clone();
        if let Some(port_value) = self.port {
            authority = format!("{}:{}", authority, port_value);
        }

        debug!("start call {}/{}", authority, method.name);

        if options.cachable {
            // TODO: GET
            // https://github.com/grpc/grpc/issues/18230
        }

        let mut headers = Headers::from_vec(vec![
            Header::new(Bytes::from_static(b":method"), Bytes::from_static(b"POST")),
            // TODO: do not allocate static
            Header::new(Bytes::from_static(b":path"), method.name.to_string()),
            Header::new(Bytes::from_static(b":authority"), authority),
            Header::new(
                Bytes::from_static(b":scheme"),
                Bytes::from_static(self.http_scheme.as_bytes()),
            ),
            Header::new(
                Bytes::from_static(b"content-type"),
                Bytes::from_static(b"application/grpc"),
            ),
            Header::new(Bytes::from_static(b"te"), Bytes::from_static(b"trailers")),
        ]);

        headers.extend(options.metadata.into_headers());

        let req_bytes = match req {
            Some(req) => {
                let mut frame = Vec::new();
                let size_estimate = match method.req_marshaller.write_size_estimate(&req) {
                    Ok(len) => len,
                    Err(e) => return Box::pin(future::err(e)),
                };
                match write_grpc_frame_cb(&mut frame, size_estimate, |frame| {
                    method.req_marshaller.write(&req, size_estimate, frame)
                }) {
                    Ok(()) => {}
                    Err(e) => return Box::pin(future::err(e)),
                }
                Some(Bytes::from(frame))
            }
            None => None,
        };

        let end_stream = req_bytes.is_some();

        //        let request_frames = {
        //            let method = method.clone();
        //            req.0
        //                .and_then(move |req| {
        //                    let grpc_frame = method.req_marshaller.write(&req)?;
        //                    Ok(Bytes::from(write_grpc_frame_to_vec(&grpc_frame)))
        //                }).map_err(|_e| httpbis::Error::Other("grpc error")) // TODO: preserve error
        //        };

        let http_future = self
            .client
            .start_request(headers, req_bytes, None, end_stream);

        let http_future = http_future.map_err(error::Error::from);

        let req_marshaller = method.req_marshaller.clone();
        let resp_marshaller = method.resp_marshaller.clone();

        Box::pin(http_future.map_ok(move |(req, resp)| {
            let grpc_req = http_req_to_grpc_frames_typed(req, req_marshaller);
            let grpc_resp = http_response_to_grpc_frames_typed(resp, resp_marshaller);
            (grpc_req, grpc_resp)
        }))

        //        Box::new(http_future
        //            .map(|(req, resp)| {
        //                let grpc_req = ClientRequestSink {
        //                    common: SinkCommon {
        //                        marshaller: method.req_marshaller,
        //                        sink: req,
        //                    }
    }

    pub fn call_unary<Req, Resp>(
        &self,
        o: RequestOptions,
        req: Req,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> SingleResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        SingleResponse::new(
            self.call_impl(o, Some(req), method)
                .and_then(|(_req, resp)| resp.single()),
        )
    }

    pub fn call_server_streaming<Req, Resp>(
        &self,
        o: RequestOptions,
        req: Req,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> StreamingResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        let f = self.call_impl(o, Some(req), method);
        StreamingResponse::new(async move {
            let (_req, resp) = f.await?;
            resp.0.await
        })
    }

    pub fn call_client_streaming<Req, Resp>(
        &self,
        o: RequestOptions,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> impl Future<Output = crate::Result<(ClientRequestSink<Req>, SingleResponse<Resp>)>>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        TryFutureExt::map_ok(self.call_impl(o, None, method), |(req, resp)| {
            (req, resp.single())
        })
    }

    pub fn call_bidi<Req, Resp>(
        &self,
        o: RequestOptions,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> impl Future<Output = crate::Result<(ClientRequestSink<Req>, StreamingResponse<Resp>)>>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, None, method)
    }
}

fn _assert_types() {
    crate::assert_types::assert_send::<Client>();
    crate::assert_types::assert_sync::<Client>();
}
