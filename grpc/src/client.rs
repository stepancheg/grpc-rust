use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;

use futures::stream::Stream;

use httpbis;
use httpbis::Header;
use httpbis::Headers;
use httpbis::HttpScheme;
use httpbis::HttpStreamAfterHeaders;
use httpbis::Service as HttpbisService;

use tls_api;

use method::MethodDescriptor;

use error::*;
use result;

use grpc_http_to_response::*;

use req::*;
use resp::*;
use proto::grpc_frame::write_grpc_frame_to_vec;

#[derive(Default, Debug, Clone)]
pub struct ClientConf {
    pub http: httpbis::ClientConf,
}

impl ClientConf {
    pub fn new() -> ClientConf {
        Default::default()
    }
}

/// gRPC client implementation.
/// Used by generated code.
pub struct Client {
    client: ::std::sync::Arc<httpbis::Client>,
    host: String,
    http_scheme: HttpScheme,
}

impl Client {
    /// Create a client connected to specified host and port.
    pub fn new_plain(host: &str, port: u16, conf: ClientConf) -> result::Result<Client> {
        let mut conf = conf;
        conf.http.thread_name = Some(
            conf.http
                .thread_name
                .unwrap_or_else(|| "grpc-client-loop".to_owned()),
        );

        httpbis::Client::new_plain(host, port, conf.http)
            .map(|client| Client {
                client: ::std::sync::Arc::new(client),
                host: host.to_owned(),
                http_scheme: HttpScheme::Http,
            }).map_err(Error::from)
    }

    /// Create a client connected to specified Unix domain socket.
    #[cfg(unix)]
    pub fn new_plain_unix(addr: &str, conf: ClientConf) -> result::Result<Client> {
        let mut conf = conf;
        conf.http.thread_name = Some(
            conf.http
                .thread_name
                .unwrap_or_else(|| "grpc-client-loop".to_owned()),
        );

        httpbis::Client::new_plain_unix(addr, conf.http)
            .map(|client| Client {
                client: ::std::sync::Arc::new(client),
                host: addr.to_owned(),
                http_scheme: HttpScheme::Http,
            }).map_err(Error::from)
    }

    /// Create a client connected to specified host and port.
    pub fn new_tls<C: tls_api::TlsConnector>(
        host: &str,
        port: u16,
        conf: ClientConf,
    ) -> result::Result<Client> {
        let mut conf = conf;
        conf.http.thread_name = Some(
            conf.http
                .thread_name
                .unwrap_or_else(|| "grpc-client-loop".to_owned()),
        );

        httpbis::Client::new_tls::<C>(host, port, conf.http)
            .map(|client| Client {
                client: ::std::sync::Arc::new(client),
                host: host.to_owned(),
                http_scheme: HttpScheme::Https,
            }).map_err(Error::from)
    }

    pub fn new_expl<C: tls_api::TlsConnector>(
        addr: &SocketAddr,
        host: &str,
        tls: httpbis::ClientTlsOption<C>,
        conf: ClientConf,
    ) -> result::Result<Client> {
        let mut conf = conf;
        conf.http.thread_name = Some(
            conf.http
                .thread_name
                .unwrap_or_else(|| "grpc-client-loop".to_owned()),
        );

        let http_scheme = tls.http_scheme();

        httpbis::Client::new_expl(addr, tls, conf.http)
            .map(|client| Client {
                client: ::std::sync::Arc::new(client),
                host: host.to_owned(),
                http_scheme: http_scheme,
            }).map_err(Error::from)
    }

    fn call_impl<Req, Resp>(
        &self,
        options: RequestOptions,
        req: StreamingRequest<Req>,
        method: Arc<MethodDescriptor<Req, Resp>>,
    ) -> StreamingResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        info!("start call {}", method.name);

        let mut headers = Headers(vec![
            Header::new(Bytes::from_static(b":method"), Bytes::from_static(b"POST")),
            Header::new(Bytes::from_static(b":path"), method.name.clone()),
            Header::new(Bytes::from_static(b":authority"), self.host.clone()),
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

        let request_frames = {
            let method = method.clone();
            req.0
                .and_then(move |req| {
                    let grpc_frame = method.req_marshaller.write(&req)?;
                    Ok(Bytes::from(write_grpc_frame_to_vec(&grpc_frame)))
                }).map_err(|_e| httpbis::Error::Other("grpc error")) // TODO: preserve error
        };

        let http_response_stream = self
            .client
            .start_request(headers, HttpStreamAfterHeaders::bytes(request_frames));

        let grpc_frames = http_response_to_grpc_frames(http_response_stream);

        grpc_frames.and_then_items(move |frame| method.resp_marshaller.read(frame))
    }

    pub fn call_unary<Req, Resp>(
        &self,
        o: RequestOptions,
        req: Req,
        method: Arc<MethodDescriptor<Req, Resp>>,
    ) -> SingleResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, StreamingRequest::once(req), method)
            .single()
    }

    pub fn call_server_streaming<Req, Resp>(
        &self,
        o: RequestOptions,
        req: Req,
        method: Arc<MethodDescriptor<Req, Resp>>,
    ) -> StreamingResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, StreamingRequest::once(req), method)
    }

    pub fn call_client_streaming<Req, Resp>(
        &self,
        o: RequestOptions,
        req: StreamingRequest<Req>,
        method: Arc<MethodDescriptor<Req, Resp>>,
    ) -> SingleResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, req, method).single()
    }

    pub fn call_bidi<Req, Resp>(
        &self,
        o: RequestOptions,
        req: StreamingRequest<Req>,
        method: Arc<MethodDescriptor<Req, Resp>>,
    ) -> StreamingResponse<Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        self.call_impl(o, req, method)
    }
}

fn _assert_types() {
    ::assert_types::assert_send::<Client>();
    ::assert_types::assert_sync::<Client>();
}
