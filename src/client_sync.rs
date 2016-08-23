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
use solicit::http::session::Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::HttpResult;
use solicit::http::Response;
use solicit::http::HttpError;

use std::net::TcpStream;

use method::MethodDescriptor;
use grpc::write_frame;
use grpc::parse_frame;
use grpc::parse_frame_completely;
use result::GrpcResult;


pub struct GrpcClient {
    host: String,
    conn: ClientConnection,
    receiver: TcpStream,
    sender: TcpStream,
}

impl GrpcClient {
    pub fn new(host: &str, port: u16) -> GrpcClient {
        let stream = CleartextConnector::with_port(host, port).connect().unwrap().0;
        let receiver = stream.try_split().unwrap();

        let conn = HttpConnection::new(HttpScheme::Http);
        let state = DefaultSessionState::<Client, _>::new();

        GrpcClient {
            host: host.to_owned(),
            conn: ClientConnection::with_connection(conn, state),
            receiver: receiver,
            sender: stream,
        }
    }

    fn new_stream<'n, 'v>(&self,
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

    fn request(&mut self,
        method: &[u8],
        path: &[u8],
        extras: &[Header],
        body: Option<Vec<u8>>)
            -> HttpResult<StreamId>
    {
        let stream = self.new_stream(method, path, extras, body);
        let stream_id = try!(self.conn.start_request(stream, &mut self.sender));
        self.conn.state.get_stream_mut(stream_id).unwrap().stream_id = Some(stream_id);

        while let SendStatus::Sent = try!(self.conn.send_next_data(&mut self.sender)) {
        }

        Ok(stream_id)
    }

    fn handle_next_frame(&mut self) -> HttpResult<()> {
        self.conn.handle_next_frame(
            &mut TransportReceiveFrame::new(&mut self.receiver),
            &mut self.sender)
    }

    fn get_response(&mut self, stream_id: StreamId) -> HttpResult<Response<'static, 'static>> {
        match self.conn.state.get_stream_ref(stream_id) {
            None => return Err(HttpError::UnknownStreamId),
            Some(_) => {}
        };
        loop {
            if let Some(stream) = self.conn.state.get_stream_ref(stream_id) {
                if stream.is_closed() {
                    return Ok(Response {
                        stream_id: stream_id,
                        headers: stream.headers.clone().unwrap(),
                        body: stream.body.clone(),
                    });
                }
            }
            try!(self.handle_next_frame());
        }
    }

    pub fn call<Req, Resp>(&mut self, req: Req, method: MethodDescriptor<Req, Resp>) -> GrpcResult<Resp> {
        let req_serialized = method.req_marshaller.write(&req);

        let mut http_req_body = Vec::new();
        write_frame(&mut http_req_body, &req_serialized);

        let stream_id = self.request(
            b"POST",
            method.name.as_bytes(),
            &[],
            Some(http_req_body))
                .unwrap();

        let http_response = self.get_response(stream_id).unwrap();

        // TODO: check code

        let resp_serialized = parse_frame_completely(&http_response.body).unwrap();

        Ok(method.resp_marshaller.read(resp_serialized))
    }
}
