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
        GrpcStream {
            default_stream: DefaultStream::new(stream_id)
        }
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        self.default_stream.new_data_chunk(data)
    }

    fn set_headers(&mut self, headers: Vec<Header>) {
        self.default_stream.set_headers(headers)
    }

    fn set_state(&mut self, state: StreamState) {
        self.default_stream.set_state(state)
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        self.default_stream.get_data_chunk(buf)
    }

    fn id(&self) -> StreamId {
        self.default_stream.id()
    }

    fn state(&self) -> StreamState {
        self.default_stream.state()
    }
}

struct GrpcSessionState {
    default_state: DefaultSessionState<GrpcStream>,
}

struct GrpcServerConnection {
    server_conn: ServerConnection<TcpStream, TcpStream>,
}

impl GrpcServerConnection {
    fn new(mut stream: TcpStream) -> GrpcServerConnection {
        let mut preface = [0; 24];
        stream.read_exact(&mut preface).unwrap();
        if &preface != b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
            panic!();
        }

        let conn = HttpConnection::<TcpStream, TcpStream>::with_stream(stream, HttpScheme::Http);
        let mut server_conn: ServerConnection<TcpStream, TcpStream> = ServerConnection::with_connection(conn, DefaultSessionState::new());

        let mut r = GrpcServerConnection {
            server_conn: server_conn,
        };

        r.server_conn.init().unwrap();
        r
    }

    fn run(&mut self) {
        panic!();
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

