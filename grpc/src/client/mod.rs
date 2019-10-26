pub(crate) mod http_request_to_grpc_frames_typed;
pub(crate) mod http_response_to_grpc_frames;
pub(crate) mod http_response_to_grpc_frames_typed;
pub(crate) mod req_sink;
pub(crate) mod types;

use bytes::Bytes;
use tokio_core::reactor::Remote;

use httpbis;
use httpbis::ClientTlsOption;
use httpbis::Header;
use httpbis::Headers;
use httpbis::HttpScheme;

use tls_api;

use method::MethodDescriptor;

use result;

use client::http_request_to_grpc_frames_typed::http_req_to_grpc_frames_typed;
use client::http_response_to_grpc_frames_typed::http_response_to_grpc_frames_typed;
use client::req_sink::ClientRequestSink;
use error;
use futures::future;
use futures::Future;
use or_static::arc::ArcOrStatic;
use proto::grpc_frame::write_grpc_frame_to_vec;
use req::*;
use resp::*;

#[derive(Default, Debug, Clone)]
pub struct ClientConf {
    pub http: httpbis::ClientConf,
}

impl ClientConf {
    pub fn new() -> ClientConf {
        Default::default()
    }
}

enum ClientBuilderType<'a> {
    Tcp { port: u16, host: &'a str },
    #[cfg(unix)]
    Unix { socket: &'a str },
}

enum Tls<T: tls_api::TlsConnector> {
    Explict(ClientTlsOption<T>),
    Implicit,
    None,
}

pub struct ClientBuilder<'a, T: tls_api::TlsConnector> {
    client_type: ClientBuilderType<'a>,
    http_scheme: HttpScheme,
    event_loop: Option<Remote>,
    pub conf: ClientConf,
    tls: Tls<T>,
}

impl<'a, T: tls_api::TlsConnector> ClientBuilder<'a, T> {
    pub fn with_event_loop(mut self, event_loop: Remote) -> Self {
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
            #[cfg(unix)]
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

    #[cfg(unix)]
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
    ) -> Box<
        Future<Item = (ClientRequestSink<Req>, StreamingResponse<Resp>), Error = error::Error>
            + Send,
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
            Some(req) => match method.req_marshaller.write(&req) {
                Ok(bytes) => {
                    // TODO: extra allocation
                    Some(Bytes::from(write_grpc_frame_to_vec(&bytes)))
                }
                Err(e) => return Box::new(future::err(e)),
            },
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

        Box::new(http_future.map(move |(req, resp)| {
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
        StreamingResponse::new(
            self.call_impl(o, Some(req), method)
                .map(|(_req, resp)| resp.0)
                .flatten(),
        )
    }

    pub fn call_client_streaming<Req, Resp>(
        &self,
        o: RequestOptions,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> impl Future<Item = (ClientRequestSink<Req>, SingleResponse<Resp>), Error = error::Error>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, None, method)
            .map(|(req, resp)| (req, resp.single()))
    }

    pub fn call_bidi<Req, Resp>(
        &self,
        o: RequestOptions,
        method: ArcOrStatic<MethodDescriptor<Req, Resp>>,
    ) -> impl Future<Item = (ClientRequestSink<Req>, StreamingResponse<Resp>), Error = error::Error>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, None, method)
    }
}

fn _assert_types() {
    ::assert_types::assert_send::<Client>();
    ::assert_types::assert_sync::<Client>();
}
