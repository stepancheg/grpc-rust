use std::net::TcpListener;
use std::net::TcpStream;

use solicit::server::SimpleServer;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::connection::HttpConnection;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::Stream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::transport::TransportStream;
use solicit::http::server::ServerConnection;

struct GrpcStream {
    default_stream: DefaultStream,
}

impl Stream for GrpcStream {
    fn new(stream_id: StreamId) -> Self {
        println!("hooray! new stream {}", stream_id);
        GrpcStream {
            default_stream: DefaultStream::new(stream_id)
        }
    }

    fn set_headers(&mut self, headers: Vec<Header>) {
        println!("hooray! headers: {:?}", headers);
        self.default_stream.set_headers(headers)
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        println!("hooray! data: {:?}", data);
        self.default_stream.new_data_chunk(data)
    }

    fn set_state(&mut self, state: StreamState) {
        println!("set_state: {:?}", state);
        self.default_stream.set_state(state)
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        panic!("get_data_chunk should not be called");
        //self.default_stream.get_data_chunk(buf)
    }

    fn id(&self) -> StreamId {
        self.default_stream.id()
    }

    fn state(&self) -> StreamState {
        self.default_stream.state()
    }
}

/*
struct GrpcSessionState {
    default_state: DefaultSessionState,
}

impl GrpcSessionState {
    fn new() -> Self {
        GrpcSessionState {
            default_state: DefaultSessionState::new(),
        }
    }
}
*/

struct GrpcServerConnection {
    server_conn: ServerConnection<TcpStream, TcpStream, DefaultSessionState<GrpcStream>>,
}

impl GrpcServerConnection {
    fn new(mut stream: TcpStream) -> GrpcServerConnection {
        let mut preface = [0; 24];
        stream.read_exact(&mut preface).unwrap();
        if &preface != b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
            panic!();
        }

        let conn = HttpConnection::<TcpStream, TcpStream>::with_stream(stream, HttpScheme::Http);
        let mut server_conn = ServerConnection::with_connection(conn, DefaultSessionState::new());

        let mut r = GrpcServerConnection {
            server_conn: server_conn,
        };

        r.server_conn.init().unwrap();
        r
    }

    fn run(&mut self) {
        loop {
            let r = self.server_conn.handle_next_frame();
            match r {
                e @ Err(..) => {
                    println!("{:?}", e);
                    return;
                }
                _ => {},
            }
        }
    }
}

pub struct GrpcServer {
    listener: TcpListener,
}

impl GrpcServer {
    pub fn new() -> GrpcServer {
        GrpcServer {
            listener: TcpListener::bind("127.0.0.1:50051").unwrap(),
        }
    }

    pub fn run(&mut self) {
        for mut stream in self.listener.incoming().map(|s| s.unwrap()) {
            println!("client connected!");
            GrpcServerConnection::new(stream).run();
        }
    }
}

