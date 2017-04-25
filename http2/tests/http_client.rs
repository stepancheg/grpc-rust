use std::net;
use std::io::Read;
use std::io::Write;
use std::str;

extern crate bytes;
extern crate httpbis;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
#[macro_use]
extern crate log;
extern crate env_logger;

use bytes::Bytes;

use futures::Future;

mod test_misc;

use test_misc::*;

use httpbis::for_test::*;
use httpbis::solicit::StreamId;
use httpbis::solicit::HttpScheme;
use httpbis::solicit::frame::FrameIR;
use httpbis::solicit::frame::settings::SettingsFrame;
use httpbis::solicit::frame::headers::HeadersFrame;
use httpbis::solicit::frame::headers::HeadersFlag;
use httpbis::solicit::frame::data::DataFrame;
use httpbis::solicit::frame::data::DataFlag;
use httpbis::solicit::frame::RawFrame;
use httpbis::solicit::connection::HttpFrame;
use httpbis::solicit::connection::HttpConnection;


struct HttpServerTester(net::TcpListener);

impl HttpServerTester {
    fn new() -> HttpServerTester {
        HttpServerTester(net::TcpListener::bind("[::1]:0".parse::<net::SocketAddr>().unwrap()).unwrap())
    }

    fn port(&self) -> u16 {
        self.0.local_addr().unwrap().port()
    }

    fn accept(&self) -> HttpConnectionTester {
        HttpConnectionTester {
            tcp: self.0.accept().unwrap().0,
            conn: HttpConnection::new(HttpScheme::Http),
        }
    }
}

struct HttpConnectionTester {
    tcp: net::TcpStream,
    conn: HttpConnection,
}

static PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

#[derive(Default)]
struct Headers(Vec<(Vec<u8>, Vec<u8>)>);

impl Headers {
    fn new() -> Headers {
        Default::default()
    }

    fn get<'a>(&'a self, name: &str) -> &'a str {
        str::from_utf8(&self.0.iter().filter(|&&(ref n, ref v)| &n[..] == name.as_bytes()).next().unwrap().1).unwrap()
    }

    fn add(&mut self, name: &str, value: &str) {
        self.0.push((name.as_bytes().to_owned(), value.as_bytes().to_owned()));
    }
}

impl HttpConnectionTester {
    fn recv_preface(&mut self) {
        let mut preface = Vec::new();
        preface.resize(PREFACE.len(), 0);
        self.tcp.read_exact(&mut preface).unwrap();
        assert_eq!(PREFACE, &preface[..]);
    }

    fn send_frame<F : FrameIR>(&mut self, frame: F) {
        self.tcp.write(&frame.serialize_into_vec()).expect("send_frame");
    }

    fn send_headers(&mut self, stream_id: StreamId, headers: Headers, end: bool) {
        let fragment = self.conn.encoder.encode(headers.0.iter().map(|&(ref n, ref v)| (&n[..], &v[..])));
        let mut headers_frame = HeadersFrame::new(fragment, stream_id);
        headers_frame.set_flag(HeadersFlag::EndHeaders);
        if end {
            headers_frame.set_flag(HeadersFlag::EndStream);
        }
        self.send_frame(headers_frame);
    }

    fn send_data(&mut self, stream_id: StreamId, data: &[u8], end: bool) {
        let mut data_frame = DataFrame::new(stream_id);
        data_frame.data = Bytes::from(data);
        if end {
            data_frame.set_flag(DataFlag::EndStream);
        }
        self.send_frame(data_frame);
    }

    fn recv_raw_frame(&mut self) -> RawFrame {
        httpbis::solicit_async::recv_raw_frame_sync(&mut self.tcp).expect("recv_raw_frame")
    }

    fn recv_frame(&mut self) -> HttpFrame {
        let raw_frame = self.recv_raw_frame();
        HttpFrame::from_raw(&raw_frame).expect("parse frame")
    }

    fn recv_frame_settings(&mut self) -> SettingsFrame {
        match self.recv_frame() {
            HttpFrame::SettingsFrame(settings) => settings,
            f => panic!("unexpected frame: {:?}", f),
        }
    }

    fn recv_frame_settings_set(&mut self) -> SettingsFrame {
        let settings = self.recv_frame_settings();
        assert!(!settings.is_ack());
        settings
    }

    fn recv_frame_settings_ack(&mut self) -> SettingsFrame {
        let settings = self.recv_frame_settings();
        assert!(settings.is_ack());
        settings
    }

    fn settings_xchg(&mut self) {
        self.send_frame(SettingsFrame::new());
        self.recv_frame_settings_set();
        self.send_frame(SettingsFrame::new_ack());
        self.recv_frame_settings_ack();
    }

    fn recv_frame_headers(&mut self) -> HeadersFrame {
        match self.recv_frame() {
            HttpFrame::HeadersFrame(headers) => headers,
            f => panic!("unexpected frame: {:?}", f),
        }
    }

    fn recv_frame_data(&mut self) -> DataFrame {
        match self.recv_frame() {
            HttpFrame::DataFrame(data) => data,
            f => panic!("unexpected frame: {:?}", f),
        }
    }

    fn recv_frame_headers_check(&mut self, stream_id: StreamId, end: bool) -> Headers {
        let headers = self.recv_frame_headers();
        assert_eq!(stream_id, headers.stream_id);
        assert_eq!(end, headers.is_end_of_stream());
        Headers(self.conn.decoder.decode(headers.header_fragment()).expect("decode"))
    }

    fn recv_frame_data_check(&mut self, stream_id: StreamId, end: bool) -> Vec<u8> {
        let data = self.recv_frame_data();
        assert_eq!(stream_id, data.stream_id);
        assert_eq!(end, data.is_end_of_stream());
        (&data.data[..]).to_vec()
    }
}

#[test]
fn stream_count_new() {
    env_logger::init().ok();

    let server = HttpServerTester::new();

    debug!("started server on {}", server.port());

    let client: HttpClient =
        HttpClient::new("::1", server.port(), false, Default::default()).expect("connect");

    let mut server_tester = server.accept();
    server_tester.recv_preface();
    server_tester.settings_xchg();

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len());

    let req = client.start_post_simple_response("/foobar", (b"xxyy"[..]).to_owned());

    let headers = server_tester.recv_frame_headers_check(1, false);
    assert_eq!("POST", headers.get(":method"));
    assert_eq!("/foobar", headers.get(":path"));

    let data = server_tester.recv_frame_data_check(1, false);
    assert_eq!(b"xxyy", &data[..]);

    let headers = server_tester.recv_frame_data_check(1, true);

    let mut resp_headers = Headers::new();
    resp_headers.add(":status", "200");
    server_tester.send_headers(1, resp_headers, false);

    server_tester.send_data(1, b"aabb", true);

    let message = req.wait().expect("r");
    assert_eq!((b"aabb"[..]).to_owned(), message.body);

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len(), "{:?}", state);
}


#[test]
fn stream_count() {
    env_logger::init().ok();

    let server = HttpServerEcho::new();

    debug!("started server on {}", server.port);

    let client: HttpClient =
        HttpClient::new("::1", server.port, false, Default::default()).expect("connect");

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len());

    let message = client.start_post_simple_response("/foobar", (b"xxyy"[..]).to_owned())
        .wait()
        .expect("r");
    assert_eq!((b"xxyy"[..]).to_owned(), message.body);

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len(), "{:?}", state);
}
