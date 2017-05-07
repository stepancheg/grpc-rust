use std::sync::Arc;
use std::net::SocketAddr;

use bytes::Bytes;

use futures;
use futures::stream;
use futures::stream::Stream;

use httpbis::HttpScheme;
use httpbis::HttpError;
use httpbis::Header;
use httpbis::Headers;
use httpbis::client::HttpClient;
use httpbis::client::ClientTlsOption;


use method::MethodDescriptor;

use error::*;
use result::*;

use httpbis::futures_misc::*;
use httpbis::client_conf::*;
use futures_grpc::*;

use grpc_frame::*;
use grpc_http_to_response::*;

use req::GrpcRequestOptions;
use resp::*;

#[derive(Default, Debug, Clone)]
pub struct GrpcClientConf {
    pub http: HttpClientConf,
}


/// gRPC client implementation.
/// Used by generated code.
pub struct GrpcClient {
    client: HttpClient,
    host: String,
    http_scheme: HttpScheme,
}

impl GrpcClient {
    /// Create a client connected to specified host and port.
    pub fn new(host: &str, port: u16, tls: bool, conf: GrpcClientConf)
        -> GrpcResult<GrpcClient>
    {
        let mut conf = conf;
        conf.http.thread_name =
            Some(conf.http.thread_name.unwrap_or_else(|| "grpc-client-loop".to_owned()));

        HttpClient::new(host, port, tls, conf.http)
            .map(|client| {
                GrpcClient {
                    client: client,
                    host: host.to_owned(),
                    http_scheme: if tls { HttpScheme::Https } else { HttpScheme::Http },
                }
            })
            .map_err(GrpcError::from)
    }

    pub fn new_expl(addr: &SocketAddr, host: &str, tls: ClientTlsOption, conf: GrpcClientConf)
        -> GrpcResult<GrpcClient>
    {
        let mut conf = conf;
        conf.http.thread_name =
            Some(conf.http.thread_name.unwrap_or_else(|| "grpc-client-loop".to_owned()));

        let http_scheme = tls.http_scheme();

        HttpClient::new_expl(addr, tls, conf.http)
            .map(|client| {
                GrpcClient {
                    client: client,
                    host: host.to_owned(),
                    http_scheme: http_scheme,
                }
            })
            .map_err(GrpcError::from)
    }

    pub fn new_resp_channel<Resp : Send + 'static>(&self)
        -> futures::Oneshot<(futures::sync::mpsc::UnboundedSender<ResultOrEof<Resp, GrpcError>>, GrpcStreamSend<Resp>)>
    {
        let (one_sender, one_receiver) = futures::oneshot();

        let (sender, receiver) = futures::sync::mpsc::unbounded();
        let receiver: GrpcStreamSend<ResultOrEof<Resp, GrpcError>> =
            Box::new(receiver.map_err(|()| GrpcError::Other("receive from resp channel")));

        let receiver: GrpcStreamSend<Resp> = Box::new(stream_with_eof_and_error(receiver, || GrpcError::Other("unexpected EOF")));

        // TODO: oneshot sender is no longer necessary as we don't use tokio queues
        one_sender.send((sender, receiver)).ok().unwrap();

        one_receiver
    }

    fn call_impl<Req, Resp, S>(&self, options: GrpcRequestOptions, req: S, method: Arc<MethodDescriptor<Req, Resp>>)
                               -> GrpcStreamingResponse<Resp>
            where
                Req : Send + 'static,
                Resp : Send + 'static,
                S : Stream<Item=Req, Error=GrpcError> + Send + 'static,
    {
        info!("start call {}", method.name);

        let mut headers = Headers(vec![
            Header::new(Bytes::from_static(b":method"), Bytes::from_static(b"POST")),
            Header::new(Bytes::from_static(b":path"), method.name.clone()),
            Header::new(Bytes::from_static(b":authority"), self.host.clone()),
            Header::new(Bytes::from_static(b":scheme"), Bytes::from_static(self.http_scheme.as_bytes())),
            Header::new(Bytes::from_static(b"content-type"), Bytes::from_static(b"application/grpc")),
        ]);

        headers.extend(options.metadata.into_headers());

        let request_frames = {
            let method = method.clone();
            req
                .and_then(move |req| {
                    let grpc_frame = method.req_marshaller.write(&req)?;
                    Ok(Bytes::from(write_grpc_frame_to_vec(&grpc_frame)))
                })
                .map_err(|e| HttpError::Other(Box::new(e)))
        };

        let http_response_stream = self.client
            .start_request(
                headers,
                Box::new(request_frames));

        let grpc_frames = http_response_to_grpc_frames(http_response_stream);

        grpc_frames.and_then_items(move |frame| method.resp_marshaller.read(&frame))
    }

    pub fn call_unary<Req, Resp>(&self, o: GrpcRequestOptions, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcSingleResponse<Resp>
            where Req: Send + 'static, Resp: Send + 'static
    {
        self.call_impl(o, stream::once(Ok(req)), method).single()
    }

    pub fn call_server_streaming<Req, Resp>(&self, o: GrpcRequestOptions, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamingResponse<Resp>
            where Req: Send + 'static, Resp: Send + 'static
    {
        self.call_impl(o, stream::once(Ok(req)), method)
    }

    pub fn call_client_streaming<Req, Resp>(&self, o: GrpcRequestOptions, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcSingleResponse<Resp>
            where Req: Send + 'static, Resp: Send + 'static
    {
        self.call_impl(o, req, method).single()
    }

    pub fn call_bidi<Req, Resp>(&self, o: GrpcRequestOptions, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamingResponse<Resp>
            where Req: Send + 'static, Resp: Send + 'static
    {
        self.call_impl(o, req, method)
    }
}
