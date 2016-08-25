use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::sync::Arc;

use futures;
use futures::Future;
use futures::stream::Stream;
use futures::oneshot;
use futures::Complete;

use futures_io;
use futures_io::TaskIo;
use futures_io::TaskIoRead;
use futures_io::TaskIoWrite;

use futures_mio;
use futures_mio::Loop;
use futures_mio::TcpStream;

use solicit::http::client::ClientConnection;
use solicit::http::client::RequestStream;
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
use solicit::http::frame::RawFrame;
use solicit::http::HttpScheme;
use solicit::http::Header;


use method::MethodDescriptor;

use error::*;
use result::*;

use futures_misc::*;
use futures_grpc::*;

use grpc::*;
use solicit_async::*;
use solicit_misc::*;


// Data sent from event loop to GrpcClient
struct LoopToClient {
    call_tx: futures_mio::Sender<Box<CallRequest>>,
    shutdown_tx: futures_mio::Sender<()>,
}

pub struct GrpcClient {
    loop_to_client: LoopToClient,
    thread_join_handle: Option<thread::JoinHandle<()>>,
}

trait CallRequest : Send {
    fn write_req(&self) -> GrpcResult<Vec<u8>>;
    fn method_name(&self) -> &str;
    fn complete(&mut self, message: &[u8]);
}

struct CallRequestTyped<Req, Resp> {
    method: Arc<MethodDescriptor<Req, Resp>>,
    req: Req,
    complete: Option<Complete<GrpcResult<Resp>>>,
}

impl<Req : Send, Resp : Send> CallRequest for CallRequestTyped<Req, Resp> {
    fn write_req(&self) -> GrpcResult<Vec<u8>> {
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

        // TODO: sync
        let socket_addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();

        let host = host.to_owned();

        let join_handle = thread::spawn(move || {
            run_client_event_loop(host, socket_addr, get_from_loop_tx);
        });

        let loop_to_client = get_from_loop_rx.recv().unwrap();

        let r = GrpcClient {
            loop_to_client: loop_to_client,
            thread_join_handle: Some(join_handle),
        };

        r
    }

    pub fn call<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>) -> GrpcFuture<Resp> {
        let (complete, oneshot) = oneshot();

        self.loop_to_client.call_tx.send(Box::new(CallRequestTyped {
            method: method,
            req: req,
            complete: Some(complete),
        })).unwrap();

        oneshot.then(|result| {
            match result {
                Ok(grpc_result) => grpc_result,
                Err(err) => Err(GrpcError::from(err)),
            }
        }).boxed()
    }
}

impl Drop for GrpcClient {
    fn drop(&mut self) {
        // ignore error because even loop may be already dead
        self.loop_to_client.shutdown_tx.send(()).ok();

        // do not ignore errors because we own event loop thread
        self.thread_join_handle.take().unwrap().join().unwrap();
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

            shared.with(|shared: &mut ClientSharedState| {
                // https://github.com/mlalic/solicit/pull/32
                let raw_frame = RawFrame::from(raw_frame.serialize());

                let mut send = VecSendFrame(Vec::new());

                // TODO: do not unwrap
                shared.conn.handle_next_frame(
                    &mut OnceReceiveFrame::new(raw_frame),
                    &mut send).unwrap();

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
    call_rx: futures_mio::Receiver<Box<CallRequest>>)
    -> GrpcFuture<()>
{
    let future = call_rx.fold((write, shared), |(write, shared), req| {
        let send_buf = shared.with(|shared: &mut ClientSharedState| {
            let mut body = Vec::new();
            write_grpc_frame(&mut body, &try!(req.write_req()));
            let path = req.method_name().as_bytes().to_vec();
            let mut stream = shared.new_stream(
                b"POST",
                &path,
                &[],
                Some(body));

            stream.stream.call = Some(req);

            let mut send_buf = VecSendFrame(Vec::new());

            // TODO: do not unwrap
            let _stream_id = shared.conn.start_request(stream, &mut send_buf).unwrap();
            while let SendStatus::Sent = shared.conn.send_next_data(&mut send_buf).unwrap() {
            }

            Ok(send_buf.0)
        });

        futures::done(send_buf)
            .and_then(|send_buf| {
                futures_io::write_all(write, send_buf)
                    .map(|(write, _)| (write, shared))
            })
    });
    future
        .map(|_| ())
        .map_err(|e| e.into())
        .boxed()
}

fn run_client_event_loop(
    host: String,
    socket_addr: SocketAddr,
    send_to_client: mpsc::Sender<LoopToClient>)
{
    let mut lp = Loop::new().unwrap();

    let (call_tx, call_rx) = lp.handle().channel();
    let (shutdown_tx, shutdown_rx) = lp.handle().channel();

    send_to_client.send(LoopToClient { call_tx: call_tx, shutdown_tx: shutdown_tx }).unwrap();

    let call_rx = call_rx.map_err(GrpcError::from);
    let shutdown_rx = shutdown_rx.map_err(GrpcError::from);

    let h = lp.handle();
    let connect = call_rx.and_then(|call_rx| {
        h.tcp_connect(&socket_addr)
            .map(|conn| (conn, call_rx))
            .map_err(GrpcError::from)
    });

    let handshake = connect.and_then(|(conn, call_rx)| {
        client_handshake(conn)
            .map(|conn| (conn, call_rx))
            .map_err(GrpcError::from)
    });

    let run_read_write = handshake.and_then(|(conn, call_rx)| {
        let (read, write) = TaskIo::new(conn).split();

        let conn = HttpConnection::new(HttpScheme::Http);
        let state = DefaultSessionState::<Client, _>::new();

        let conn = ClientConnection::with_connection(conn, state);

        let shared_for_read = TaskDataMutex::new(ClientSharedState {
            host: host,
            conn: conn,
        });
        let shared_for_write = shared_for_read.clone();
        run_read(read, shared_for_read)
            .join(run_write(write, shared_for_write, call_rx))
                .map_err(GrpcError::from)
    });

    let shutdown = shutdown_rx.and_then(|shutdown_rx| {
        shutdown_rx.into_future()
            .map_err(|_| GrpcError::Other("error in shutdown channel"))
            .and_then(|_| {
                let result: Result<(), _> = Err(GrpcError::Other("shutdown"));
                result
            })
    });

    let done = run_read_write.join(shutdown);

    lp.run(done).ok();
}
