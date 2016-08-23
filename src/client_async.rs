use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::iter::repeat;
use std::cell::RefCell;

use futures::Future;
use futures::stream;
use futures::stream::Stream;
use futures::stream::channel;
use futures::stream::Sender;
use futures::stream::Receiver;
use futures::oneshot;
use futures::Oneshot;
use futures::Complete;
use futures::task::TaskData;
use futures::done;
use futures::empty;

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
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
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

use http2_async::*;
use io_misc::*;


pub struct GrpcClientAsync {
    tx: SyncSender<Box<CallRequest>, GrpcError>,
}

trait CallRequest : Send {
    fn write_req(&self) -> Vec<u8>;
    fn method_name(&self) -> &str;
}

struct CallRequestTyped<Req, Resp> {
    method: MethodDescriptor<Req, Resp>,
    req: Req,
    complete: Complete<Resp>, // TODO: GrpcError
}

impl<Req : Send, Resp : Send> CallRequest for CallRequestTyped<Req, Resp> {
    fn write_req(&self) -> Vec<u8> {
        self.method.req_marshaller.write(&self.req)
    }

    fn method_name(&self) -> &str {
        &self.method.name
    }
}

impl GrpcClientAsync {
    pub fn new(host: &str, port: u16) -> GrpcClientAsync {

        let (tx, rx) = channel_sync_sender();

        // TODO: sync
        let socket_addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        thread::spawn(move || {
            run_event_loop(socket_addr, rx);
        });

        let r = GrpcClientAsync {
            tx: tx,
        };

        r
    }

    pub fn call<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: MethodDescriptor<Req, Resp>) -> GrpcFuture<Resp> {
        let (complete, oneshot) = oneshot();

        self.tx.send(Ok(Box::new(CallRequestTyped {
            method: method,
            req: req,
            complete: complete,
        })));

        oneshot.map_err(|e| GrpcError::Other).boxed()
    }
}

struct ReadWriteSharedState {
    host: String,
    conn: ClientConnection,
}

impl ReadWriteSharedState {
    fn new_stream<'n, 'v>(
        &self,
        method: &'v [u8],
        path: &'v [u8],
        extras: &[Header<'n, 'v>],
        body: Option<Vec<u8>>)
            -> RequestStream<'n, 'v, DefaultStream>
    {
        let mut stream = DefaultStream::new();
        match body {
            Some(body) => stream.set_full_data(body),
            None => stream.close_local(),
        };

        let mut headers: Vec<Header> = vec![
            Header::new(b":method", method),
            Header::new(b":path", path),
            Header::new(b":authority", self.host.clone().into_bytes()),
            Header::new(b":scheme", self.conn.scheme().as_bytes().to_vec()),
        ];
        // The clone is lightweight if the original Header was just borrowing something; it's a
        // deep copy if it was already owned. Consider requiring that this method gets an iterator
        // of Headers...
        headers.extend(extras.iter().cloned());

        RequestStream {
            headers: headers,
            stream: stream,
        }
    }
}

fn run_read(read: TaskIoRead<TcpStream>, shared: TaskData<RefCell<ReadWriteSharedState>>) -> GrpcFuture<()> {
    empty().boxed()
}

fn run_write(
    write: TaskIoWrite<TcpStream>,
    shared: TaskData<RefCell<ReadWriteSharedState>>,
    rx: Receiver<Box<CallRequest>, GrpcError>)
        -> GrpcFuture<()>
{
    // TODO: somewhat weird
    let shareds = stream::iter(repeat(shared).map(|s| Ok(s)));;
    let stream = rx.zip(shareds).and_then(|(req, shared)| {
        let stream_id = shared.with(|shared| {
            let stream = shared.borrow().new_stream(
                b"POST",
                req.method_name().as_bytes(),
                &[],
                Some(req.write_req()));
            let stream_id = shared.borrow_mut().conn.state.insert_outgoing(stream.stream);

            //let headers_fragment = shared.borrow_mut().conn.conn
            //                           .encoder
            //                           .encode(stream.headers.into().iter().map(|h| (h.name(), h.value())));
            // For now, sending header fragments larger than 16kB is not supported
            // (i.e. the encoded representation cannot be split into CONTINUATION
            // frames).
//            let mut frame = HeadersFrame::new(headers_fragment, stream_id);
//            frame.set_flag(HeadersFlag::EndHeaders);
//
//            if end_stream == EndStream::Yes {
//                frame.set_flag(HeadersFlag::EndStream);
//            }

            stream_id;
        });

        // TODO: I AM HERE
        //self.conn.sender(sender).send_headers(req.headers, stream_id, end_stream)
        //shared.borrow_mut().conn.state.get_stream_mut(stream_id).unwrap().stream_id = Some(stream_id);
        //while let SendStatus::Sent = try!(self.conn.send_next_data(&mut self.sender)) {
        //}

        empty::<(), GrpcError>()
    });
    stream.fold((), |_, _| done::<(), GrpcError>(Ok(()))).boxed()
}

fn run_event_loop(socket_addr: SocketAddr, rx: Receiver<Box<CallRequest>, GrpcError>) {
    let mut lp = Loop::new().unwrap();

    let connect = lp.handle().tcp_connect(&socket_addr);

    let initial = connect.and_then(|conn| initial(conn).map_err(|_| io_error_other("todo")));

    let done = initial.and_then(|conn| {
        let (read, write) = TaskIo::new(conn).split();

        let conn = HttpConnection::new(HttpScheme::Http);
        let state = DefaultSessionState::<Client, _>::new();

        let conn = ClientConnection::with_connection(conn, state);

        let shared_for_read = TaskData::new(RefCell::new(ReadWriteSharedState {
            host: "localhost".to_string(), // TODO
            conn: conn,
        }));
        let shared_for_write = shared_for_read.clone();
        run_read(read, shared_for_read)
            .join(run_write(write, shared_for_write, rx))
                .map_err(|e| io_error_other("todo"))
    });

    lp.run(done).unwrap();
}
