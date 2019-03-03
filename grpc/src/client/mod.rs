pub(crate) mod http_request_to_grpc_frames_typed;
pub(crate) mod http_response_to_grpc_frames;
pub(crate) mod http_response_to_grpc_frames_typed;
pub(crate) mod req_sink;
pub(crate) mod types;

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;

use httpbis;
use httpbis::Header;
use httpbis::Headers;
use httpbis::HttpScheme;

use tls_api;

use method::MethodDescriptor;

use error::*;
use result;

use client::http_request_to_grpc_frames_typed::http_req_to_grpc_frames_typed;
use client::http_response_to_grpc_frames_typed::http_response_to_grpc_frames_typed;
use client::req_sink::ClientRequestSink;
use error;
use futures::future;
use futures::Future;
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
                port: Some(port.to_owned()),
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
                port: None,
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
                port: Some(port.to_owned()),
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
                port: Some(addr.port()),
            }).map_err(Error::from)
    }

    fn call_impl<Req, Resp>(
        &self,
        options: RequestOptions,
        req: Option<Req>,
        method: Arc<MethodDescriptor<Req, Resp>>,
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

        info!("start call {}/{}", authority, method.name);

        let mut headers = Headers::from_vec(vec![
            Header::new(Bytes::from_static(b":method"), Bytes::from_static(b"POST")),
            Header::new(Bytes::from_static(b":path"), method.name.clone()),
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
        method: Arc<MethodDescriptor<Req, Resp>>,
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
        method: Arc<MethodDescriptor<Req, Resp>>,
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
        method: Arc<MethodDescriptor<Req, Resp>>,
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
        method: Arc<MethodDescriptor<Req, Resp>>,
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
