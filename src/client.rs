use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::iter::repeat;

use futures::Future;
use futures::stream;
use futures::stream::Stream;
use futures::stream::channel;
use futures::stream::Sender;
use futures::stream::Receiver;
use futures::oneshot;
use futures::Complete;

use futures_io;
use futures_io::write_all;
use futures_io::TaskIo;
use futures_io::TaskIoRead;
use futures_io::TaskIoWrite;

use futures_mio::Loop;
use futures_mio::TcpStream;

use futures_grpc::GrpcFuture;
use futures_grpc::GrpcStream;

use solicit::http::client::ClientConnection;
use solicit::http::client::CleartextConnector;
use solicit::http::client::ClientStream;
use solicit::http::client::HttpConnect;
use solicit::http::client::RequestStream;
use solicit::http::transport::TransportStream;
use solicit::http::transport::TransportReceiveFrame;
use solicit::http::session::Client;
use solicit::http::session::SessionState;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::connection::HttpFrame;
use solicit::http::frame::RawFrame;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::HttpResult;
use solicit::http::Response;
use solicit::http::HttpError;


use channel_sync_sender::SyncSender;
use channel_sync_sender::channel_sync_sender;
use method::MethodDescriptor;
use result::GrpcError;

use futures_misc::*;

use grpc::*;
use solicit_async::*;
use solicit_misc::*;
use io_misc::*;
use misc::*;


pub struct GrpcClient {
    tx: SyncSender<Box<CallRequest>, GrpcError>,
}

trait CallRequest : Send {
    fn write_req(&self) -> Vec<u8>;
    fn method_name(&self) -> &str;
    fn complete(&mut self, message: &[u8]);
}

struct CallRequestTyped<Req, Resp> {
    method: MethodDescriptor<Req, Resp>,
    req: Req,
    complete: Option<Complete<Resp>>, // TODO: GrpcError
}

impl<Req : Send, Resp : Send> CallRequest for CallRequestTyped<Req, Resp> {
    fn write_req(&self) -> Vec<u8> {
        self.method.req_marshaller.write(&self.req)
    }

    fn method_name(&self) -> &str {
        &self.method.name
    }

    fn complete(&mut self, message: &[u8]) {
        self.complete.take().unwrap().complete(self.method.resp_marshaller.read(message));
    }
}

impl GrpcClient {
    pub fn new(host: &str, port: u16) -> GrpcClient {

        // TODO: must not be sync
        let (tx, rx) = channel_sync_sender();

        // TODO: sync
        let socket_addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        thread::spawn(move || {
            run_client_event_loop(socket_addr, rx);
        });

        let r = GrpcClient {
            tx: tx,
        };

        r
    }

    pub fn call<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: MethodDescriptor<Req, Resp>) -> GrpcFuture<Resp> {
        let (complete, oneshot) = oneshot();

        self.tx.send(Ok(Box::new(CallRequestTyped {
            method: method,
            req: req,
            complete: Some(complete),
        })));

        oneshot.map_err(|e| GrpcError::Other("call")).boxed()
    }
}

struct GrpcHttp2ClientStream {
    stream: DefaultStream,
    call: Option<Box<CallRequest>>,
}

impl GrpcHttp2ClientStream {
    fn new() -> GrpcHttp2ClientStream {
        GrpcHttp2ClientStream {
            stream: DefaultStream::new(),
            call: None,
        }
    }
}

impl solicit_Stream for GrpcHttp2ClientStream {
    fn new_data_chunk(&mut self, data: &[u8]) {
        self.stream.new_data_chunk(data)
    }

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) {
        self.stream.set_headers(headers)
    }

    fn set_state(&mut self, state: StreamState) {
        //println!("set_state: {:?}", state);
        if state == StreamState::Closed {
            //println!("response body: {:?}", BsDebug(&self.stream.body));
            let message_serialized = parse_grpc_frame_completely(&self.stream.body).unwrap();
            self.call.as_mut().unwrap().complete(message_serialized);
        }
        self.stream.set_state(state)
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        self.stream.get_data_chunk(buf)
    }

    fn state(&self) -> StreamState {
        self.stream.state()
    }
}

struct ClientSharedState {
    host: String,
    conn: ClientConnection<DefaultSessionState<Client, GrpcHttp2ClientStream>>,
}

unsafe impl Sync for ClientSharedState {}

impl ClientSharedState {
    fn new_stream<'n, 'v>(
        &self,
        method: &'v [u8],
        path: &'v [u8],
        extras: &[Header<'n, 'v>],
        body: Option<Vec<u8>>)
            -> RequestStream<'n, 'v, GrpcHttp2ClientStream>
    {
        let mut stream = GrpcHttp2ClientStream::new();
        match body {
            Some(body) => stream.stream.set_full_data(body),
            None => stream.close_local(),
        };

        let mut headers: Vec<Header> = vec![
            Header::new(b":method", method),
            Header::new(b":path", path),
            Header::new(b":authority", self.host.clone().into_bytes()),
            Header::new(b":scheme", self.conn.scheme().as_bytes().to_vec()),
        ];

        headers.extend(extras.iter().cloned());

        RequestStream {
            headers: headers,
            stream: stream,
        }
    }
}

fn run_read(
    read: TaskIoRead<TcpStream>,
    shared: TaskDataMutex<ClientSharedState>)
        -> GrpcFuture<()>
{
    let stream = stream_repeat(());

    let future = stream.fold((read, shared), |(read, shared), _| {
        recv_raw_frame(read).map(|(read, raw_frame)| {

            shared.with(|shared| {
                // https://github.com/mlalic/solicit/pull/32
                let raw_frame = RawFrame::from(raw_frame.serialize());

                let mut send = VecSendFrame(Vec::new());

                shared.conn.handle_next_frame(
                    &mut OnceReceiveFrame::new(raw_frame),
                    &mut send);

                // TODO: process send

                shared.conn.state.get_closed();
            });

            (read, shared)
        })
    });

    future.map(|_| ()).boxed()
}

fn run_write(
    write: TaskIoWrite<TcpStream>,
    shared: TaskDataMutex<ClientSharedState>,
    rx: Receiver<Box<CallRequest>, GrpcError>)
    -> GrpcFuture<()>
{
    let future = rx.fold((write, shared), |(write, shared), req| {
        let buf = shared.with(|shared| {
            let mut body = Vec::new();
            write_grpc_frame(&mut body, &req.write_req());
            let path = req.method_name().as_bytes().to_vec();
            let mut stream = shared.new_stream(
                b"POST",
                &path,
                &[],
                Some(body));

            stream.stream.call = Some(req);

            let mut buf = VecSendFrame(Vec::new());

            let stream_id = shared.conn.start_request(stream, &mut buf).unwrap();
            while let SendStatus::Sent = shared.conn.send_next_data(&mut buf).unwrap() {
            }

            buf.0
        });

        //println!("write_all {:?}", BsDebug(&buf));

        futures_io::write_all(write, buf)
            .map(|(write, _)| (write, shared))
            //.map(|x| { println!("written"); x })
    });
    future.map(|_| ()).boxed()
}

fn run_client_event_loop(socket_addr: SocketAddr, rx: Receiver<Box<CallRequest>, GrpcError>) {
    let mut lp = Loop::new().unwrap();

    let connect = lp.handle().tcp_connect(&socket_addr);

    let handshake = connect.and_then(|conn| {
        client_handshake(conn).map_err(|_| io_error_other("connect"))
    });

    let done = handshake.and_then(|conn| {
        let (read, write) = TaskIo::new(conn).split();

        let conn = HttpConnection::new(HttpScheme::Http);
        let state = DefaultSessionState::<Client, _>::new();

        let conn = ClientConnection::with_connection(conn, state);

        let shared_for_read = TaskDataMutex::new(ClientSharedState {
            host: "localhost".to_string(), // TODO
            conn: conn,
        });
        let shared_for_write = shared_for_read.clone();
        run_read(read, shared_for_read)
            .join(run_write(write, shared_for_write, rx))
                .map_err(|e| e.into())
    });

    lp.run(done).unwrap();
}
