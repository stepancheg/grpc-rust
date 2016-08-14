use std::net::TcpListener;
use std::net::TcpStream;
use std::fmt;
use std::io::Cursor;
use std::io::Read;
use std::convert::From;

use solicit::server::SimpleServer;
use solicit::http::server::StreamFactory;
use solicit::http::server::ServerConnection;
use solicit::http::server::ServerSession;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::HeaderPart;
use solicit::http::OwnedHeader;
use solicit::http::HttpResult;
use solicit::http::priority::SimplePrioritizer;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::SendStatus;
use solicit::http::connection::SendFrame;
use solicit::http::connection::DataChunk;
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
    stream_id: Option<StreamId>,
    headers: Option<Vec<Header<'static, 'static>>>,
    body: Vec<u8>,
    state: StreamState,
    data: Option<Cursor<Vec<u8>>>,

    req_buf: Vec<u8>,
    resp_buf: Vec<u8>,
    service_definition: ServerServiceDefinition,
    path: String,
}

impl GrpcStream {
    fn with_id(stream_id: StreamId) -> Self {
        println!("new stream {}", stream_id);
        GrpcStream {
            stream_id: Some(stream_id),
            headers: None,
            body: Vec::new(),
            state: StreamState::Open,
            data: None,
            req_buf: Vec::new(),
            resp_buf: Vec::new(),
            service_definition: ServerServiceDefinition::new(Vec::new()),
            path: String::new(),
        }
    }

    fn process_buf(&mut self) {
        loop {
            let (r, pos) = match grpc::parse_frame(&self.req_buf) {
                Some((frame, pos)) => {
                    let r = self.service_definition.handle_method(&self.path, frame);
                    (r, pos)
                }
                None => return,
            };

            self.req_buf.drain(..pos);
            grpc::write_frame(&mut self.resp_buf, &r);
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
        self.headers = Some(headers.into_iter().map(|h| {
            let owned: OwnedHeader = h.into();
            owned.into()
        }).collect());
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        println!("hooray! data chunk: {:?} in stream {:?}", data, self.stream_id);
        self.req_buf.extend(data);
        self.process_buf();
    }

    fn set_state(&mut self, state: StreamState) {
        println!("set_state: {:?}", state);
        self.state = state;
        println!("s: {:?}", BsDebug(&self.body));
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        println!("get_data_chunk");
        if self.is_closed_local() {
            return Err(StreamDataError::Closed);
        }
        let chunk = match self.data.as_mut() {
            // No data associated to the stream, but it's open => nothing available for writing
            None => StreamDataChunk::Unavailable,
            Some(d) =>  {
                // For the `Vec`-backed reader, this should never fail, so unwrapping is
                // fine.
                let read = d.read(buf).unwrap();
                if (d.position() as usize) == d.get_ref().len() {
                    StreamDataChunk::Last(read)
                } else {
                    StreamDataChunk::Chunk(read)
                }
            }
        };
        // Transition the stream state to locally closed if we've extracted the final data chunk.
        match chunk {
            StreamDataChunk::Last(_) => self.close_local(),
            _ => {},
        };

        Ok(chunk)
    }

    fn state(&self) -> StreamState {
        self.state
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
    conn: HttpConnection,
    factory: GrpcStreamFactory,
    state: DefaultSessionState<Server, GrpcStream>,
    receiver: TcpStream,
    sender: TcpStream,
}

impl GrpcServerConnection {
    fn new(mut stream: TcpStream) -> GrpcServerConnection {
        let mut preface = [0; 24];

        (&mut stream as &mut Read).read_exact(&mut preface).unwrap();
        if &preface != b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
            panic!();
        }

        let conn = HttpConnection::new(HttpScheme::Http);

		let mut xx: TcpStream = stream.try_split().unwrap();
        let mut r = GrpcServerConnection {
            conn: conn,
            state: DefaultSessionState::<Server, _>::new(),
            receiver: xx,
            sender: stream,
            factory: GrpcStreamFactory,
        };

        //r.server_conn.init().unwrap();
        r
    }
    
    fn handle_requests(&mut self) -> HttpResult<Vec<(StreamId, Vec<u8>)>> {
        Ok(self.state.iter().flat_map(|(&id, s)| {
            if s.resp.is_empty() {
                None
            } else {
                Some((id, s.resp.clone()))
            }        
        }).collect())
    }
    
    fn prepare_responses(&mut self, responses: Vec<(StreamId, Vec<u8>)>) -> HttpResult<()> {
        for r in responses {
            try!(self.conn.sender(&mut self.sender).send_headers(
                vec![
                	Header::new(b":status", b"200"),
                	Header::new(&b"content-type"[..], &b"application/grpc"[..]),
                ],
                r.0,
                EndStream::No));
            
            try!(self.conn.sender(&mut self.sender).send_data(DataChunk::new_borrowed(&r.1[..], r.0, EndStream::No)));
            
            try!(self.conn.sender(&mut self.sender).send_headers(
                vec![
                	Header::new(&b"grpc-status"[..], b"0"),
                ],
                r.0,
                EndStream::Yes));
        }
        Ok(())
    }

    fn send_next_data(&mut self) -> HttpResult<SendStatus> {
        const MAX_CHUNK_SIZE: usize = 8 * 1024;
        let mut buf = [0; MAX_CHUNK_SIZE];

        // TODO: Additionally account for the flow control windows.
        let mut prioritizer = SimplePrioritizer::new(&mut self.state, &mut buf);

        self.conn.sender(&mut self.sender).send_next_data(&mut prioritizer)
    }

    fn flush_streams(&mut self) -> HttpResult<()> {
        while let SendStatus::Sent = try!(self.send_next_data()) {}

        Ok(())
    }

    fn reap_streams(&mut self) {
        // Moves the streams out of the state and then drops them
        let closed = self.state.get_closed();
        println!("closed: {:?}", closed.iter().map(|s| s.stream_id).collect::<Vec<_>>());
    }

    pub fn handle_next_frame(&mut self) -> HttpResult<()> {
        let mut rx = TransportReceiveFrame::new(&mut self.receiver);
        let mut session = ServerSession::new(&mut self.state, &mut self.factory, &mut self.sender);
        self.conn.handle_next_frame(&mut rx, &mut session)
    }

    fn handle_next(&mut self) -> HttpResult<()> {
        try!(self.handle_next_frame());
        
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

