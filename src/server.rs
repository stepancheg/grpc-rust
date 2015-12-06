use std::net::TcpListener;
use std::net::TcpStream;

use solicit::server::SimpleServer;
use solicit::http::server::StreamFactory;
use solicit::http::server::ServerConnection;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::HttpResult;
use solicit::http::connection::HttpConnection;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::Stream;
use solicit::http::session::Server;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::transport::TransportStream;
use solicit::http::transport::TransportReceiveFrame;

struct GrpcStream {
    default_stream: DefaultStream,
}

impl GrpcStream {
    fn with_id(stream_id: StreamId) -> Self {
        println!("new stream {}", stream_id);
        GrpcStream {
            default_stream: DefaultStream::with_id(stream_id)
        }
    }
}

impl Stream for GrpcStream {
    fn set_headers(&mut self, headers: Vec<Header>) {
        println!("headers: {:?}", headers);
        self.default_stream.set_headers(headers)
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        println!("hooray! data: {:?}", data);
        self.default_stream.new_data_chunk(data)
    }

    fn set_state(&mut self, state: StreamState) {
        println!("set_state: {:?}", state);
        self.default_stream.set_state(state);
        println!("{:?}", self.default_stream.body);
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        panic!("get_data_chunk should not be called");
        //self.default_stream.get_data_chunk(buf)
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

struct GrpcStreamFactory;

impl StreamFactory for GrpcStreamFactory {
	type Stream = GrpcStream;
	
	fn create(&mut self, id: StreamId) -> GrpcStream {
		GrpcStream::with_id(id)
	}
}

struct GrpcServerConnection {
    server_conn: ServerConnection<GrpcStreamFactory, DefaultSessionState<Server, GrpcStream>>,
    receiver: TcpStream,
    sender: TcpStream,
}

impl GrpcServerConnection {
    fn new(mut stream: TcpStream) -> GrpcServerConnection {
        let mut preface = [0; 24];
        stream.read_exact(&mut preface).unwrap();
        if &preface != b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
            panic!();
        }

        let conn = HttpConnection::new(HttpScheme::Http);
        let mut server_conn = ServerConnection::with_connection(conn, DefaultSessionState::<Server, _>::new(), GrpcStreamFactory);

		let mut xx: TcpStream = stream.try_split().unwrap();
        let mut r = GrpcServerConnection {
            server_conn: server_conn,
            receiver: xx,
            sender: stream,
        };

        //r.server_conn.init().unwrap();
        r
    }

    fn handle_next(&mut self) -> HttpResult<()> {
        try!(self.server_conn.handle_next_frame(
            &mut TransportReceiveFrame::new(&mut self.receiver),
            &mut self.sender));
        //let responses = try!(self.handle_requests());
        //try!(self.prepare_responses(responses));
        //try!(self.flush_streams());
        //try!(self.reap_streams());

        Ok(())
    }

    fn run(&mut self) {
        loop {
            let r = self.handle_next();
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

