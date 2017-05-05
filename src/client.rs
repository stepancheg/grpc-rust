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

    pub fn call_impl<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
    {
        info!("start call {}", method.name);

        let host = self.host.clone();
        let http_scheme = self.http_scheme.clone();

        let headers = Headers(vec![
            Header::new(":method", "POST"),
            Header::new(":path", method.name.clone()),
            Header::new(":authority", host.clone()),
            Header::new(":scheme", http_scheme.as_bytes()),
            Header::new("content-type", "application/grpc"),
        ]);

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

        let grpc_frames = GrpcFrameFromHttpFramesStreamResponse::new(http_response_stream);

        let grpc_messages = grpc_frames.and_then(move |frame| method.resp_marshaller.read(&frame));

        Box::new(grpc_messages)
    }

    pub fn call_unary<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFutureSend<Resp>
    {
        Box::new(stream_single(self.call_impl(Box::new(stream::once(Ok(req))), method)))
    }

    pub fn call_server_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
    {
        self.call_impl(stream::once(Ok(req)).boxed(), method)
    }

    pub fn call_client_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFutureSend<Resp>
    {
        Box::new(stream_single(self.call_impl(req, method)))
    }

    pub fn call_bidi<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
    {
        self.call_impl(req, method)
    }
}
