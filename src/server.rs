use std::net::TcpListener;
use std::net::TcpStream;
use std::fmt;

use solicit::server::SimpleServer;
use solicit::http::server::StreamFactory;
use solicit::http::server::ServerConnection;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::HttpResult;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::SendStatus;
use solicit::http::session::SessionState;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::Stream;
use solicit::http::session::Server;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::transport::TransportStream;
use solicit::http::transport::TransportReceiveFrame;

use grpc;
use method::ServerServiceDefinition;

struct BsDebug<'a>(&'a [u8]);

impl<'a> fmt::Debug for BsDebug<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "b\""));
        let u8a: &[u8] = self.0;
        for &c in u8a {
            // ASCII printable
            if c >= 0x20 && c < 0x7f {
                try!(write!(fmt, "{}", c as char));
            } else {
                try!(write!(fmt, "\\x{:02x}", c));
            }
        }
        try!(write!(fmt, "\""));
    	Ok(())
    }
}

struct HeaderDebug<'a>(&'a Header<'a, 'a>);

impl<'a> fmt::Debug for HeaderDebug<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
	    write!(fmt, "Header {{ name: {:?}, value: {:?} }}",
	    	BsDebug(self.0.name()), BsDebug(self.0.value()))
	}
}

struct GrpcStream {
    default_stream: DefaultStream,
    buf: Vec<u8>,
    resp: Vec<Vec<u8>>,
    service_definition: ServerServiceDefinition,
    path: String,
}

impl GrpcStream {
    fn with_id(stream_id: StreamId) -> Self {
        println!("new stream {}", stream_id);
        GrpcStream {
            default_stream: DefaultStream::with_id(stream_id),
            buf: Vec::new(),
            resp: Vec::new(),
            service_definition: ServerServiceDefinition::new(Vec::new()),
            path: String::new(),
        }
    }

    fn process_buf(&mut self) {
        loop {
            let (r, pos) = match grpc::parse_frame(&self.buf) {
                Some((frame, pos)) => {
                    let r = self.service_definition.handle_method(&self.path, frame);
                    (r, pos)
                }
                None => return,
            };

            self.buf.drain(..pos);
            self.resp.push(r);
        }
    }
}

impl Stream for GrpcStream {
    fn set_headers(&mut self, headers: Vec<Header>) {
        for h in &headers {
            if h.name() == b":path" {
                self.path = String::from_utf8(h.value().to_owned()).unwrap();
            }
        }
        println!("headers: {:?}", headers.iter().map(|h| HeaderDebug(h)).collect::<Vec<_>>());
        self.default_stream.set_headers(headers)
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        println!("hooray! data: {:?}", data);
        self.buf.extend(data);
        self.process_buf();
        println!("{:?}", grpc::parse_frame(data));
        self.default_stream.new_data_chunk(data)
    }

    fn set_state(&mut self, state: StreamState) {
        println!("set_state: {:?}", state);
        self.default_stream.set_state(state);
        println!("s: {:?}", BsDebug(&self.default_stream.body));
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        println!("get_data_chunk");
        self.default_stream.get_data_chunk(buf)
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
    
    fn handle_requests(&mut self) -> HttpResult<Vec<(StreamId, Vec<u8>)>> {
        Ok(self.server_conn.state.iter().flat_map(|(&id, s)| {
            if s.resp.is_empty() {
                None
            } else {
                Some((id, s.resp[0].clone()))
            }        
        }).collect())
    }
    
    fn prepare_responses(&mut self, responses: Vec<(StreamId, Vec<u8>)>) -> HttpResult<()> {
        for r in responses {
            try!(self.server_conn.start_response(
                Vec::new(),
                r.0,
                EndStream::No,
                &mut self.sender    
            ));
            let mut stream = self.server_conn.state.get_stream_mut(r.0).unwrap();
            stream.default_stream.set_full_data(r.1);
        }
        Ok(())
    }

    fn flush_streams(&mut self) -> HttpResult<()> {
        while let SendStatus::Sent = try!(self.server_conn.send_next_data(&mut self.sender)) {}

        Ok(())
    }

    fn reap_streams(&mut self) {
        // Moves the streams out of the state and then drops them
        let _ = self.server_conn.state.get_closed();
    }

    fn handle_next(&mut self) -> HttpResult<()> {
        try!(self.server_conn.handle_next_frame(
            &mut TransportReceiveFrame::new(&mut self.receiver),
            &mut self.sender));
        
        let responses = try!(self.handle_requests());

		try!(self.prepare_responses(responses));

        try!(self.flush_streams());
        self.reap_streams();

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

