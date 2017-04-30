#![allow(dead_code)]

use std::io::Write;
use std::io::Read;

use std::str;
use std::net;

use bytes::Bytes;

use httpbis;
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


pub struct HttpServerTester(net::TcpListener);

impl HttpServerTester {
    pub fn new() -> HttpServerTester {
        HttpServerTester(net::TcpListener::bind("[::1]:0".parse::<net::SocketAddr>().unwrap()).unwrap())
    }

    pub fn port(&self) -> u16 {
        self.0.local_addr().unwrap().port()
    }

    pub fn accept(&self) -> HttpConnectionTester {
        HttpConnectionTester {
            tcp: self.0.accept().unwrap().0,
            conn: HttpConnection::new(HttpScheme::Http),
        }
    }
}

pub struct HttpConnectionTester {
    tcp: net::TcpStream,
    conn: HttpConnection,
}

static PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

#[derive(Default)]
pub struct Headers(Vec<(Vec<u8>, Vec<u8>)>);

impl Headers {
    pub fn new() -> Headers {
        Default::default()
    }

    pub fn get<'a>(&'a self, name: &str) -> &'a str {
        str::from_utf8(&self.0.iter().filter(|&&(ref n, ref _v)| &n[..] == name.as_bytes()).next().unwrap().1).unwrap()
    }

    pub fn add(&mut self, name: &str, value: &str) {
        self.0.push((name.as_bytes().to_owned(), value.as_bytes().to_owned()));
    }
}

impl HttpConnectionTester {
    pub fn recv_preface(&mut self) {
        let mut preface = Vec::new();
        preface.resize(PREFACE.len(), 0);
        self.tcp.read_exact(&mut preface).unwrap();
        assert_eq!(PREFACE, &preface[..]);
    }

    pub fn send_frame<F : FrameIR>(&mut self, frame: F) {
        self.tcp.write(&frame.serialize_into_vec()).expect("send_frame");
    }

    pub fn send_headers(&mut self, stream_id: StreamId, headers: Headers, end: bool) {
        let fragment = self.conn.encoder.encode(headers.0.iter().map(|&(ref n, ref v)| (&n[..], &v[..])));
        let mut headers_frame = HeadersFrame::new(fragment, stream_id);
        headers_frame.set_flag(HeadersFlag::EndHeaders);
        if end {
            headers_frame.set_flag(HeadersFlag::EndStream);
        }
        self.send_frame(headers_frame);
    }

    pub fn send_data(&mut self, stream_id: StreamId, data: &[u8], end: bool) {
        let mut data_frame = DataFrame::new(stream_id);
        data_frame.data = Bytes::from(data);
        if end {
            data_frame.set_flag(DataFlag::EndStream);
        }
        self.send_frame(data_frame);
    }

    pub fn recv_raw_frame(&mut self) -> RawFrame {
        httpbis::solicit_async::recv_raw_frame_sync(&mut self.tcp).expect("recv_raw_frame")
    }

    pub fn recv_frame(&mut self) -> HttpFrame {
        let raw_frame = self.recv_raw_frame();
        HttpFrame::from_raw(&raw_frame).expect("parse frame")
    }

    pub fn recv_frame_settings(&mut self) -> SettingsFrame {
        match self.recv_frame() {
            HttpFrame::SettingsFrame(settings) => settings,
            f => panic!("unexpected frame: {:?}", f),
        }
    }

    pub fn recv_frame_settings_set(&mut self) -> SettingsFrame {
        let settings = self.recv_frame_settings();
        assert!(!settings.is_ack());
        settings
    }

    pub fn recv_frame_settings_ack(&mut self) -> SettingsFrame {
        let settings = self.recv_frame_settings();
        assert!(settings.is_ack());
        settings
    }

    pub fn settings_xchg(&mut self) {
        self.send_frame(SettingsFrame::new());
        self.recv_frame_settings_set();
        self.send_frame(SettingsFrame::new_ack());
        self.recv_frame_settings_ack();
    }

    pub fn recv_frame_headers(&mut self) -> HeadersFrame {
        match self.recv_frame() {
            HttpFrame::HeadersFrame(headers) => headers,
            f => panic!("unexpected frame: {:?}", f),
        }
    }

    pub fn recv_frame_data(&mut self) -> DataFrame {
        match self.recv_frame() {
            HttpFrame::DataFrame(data) => data,
            f => panic!("unexpected frame: {:?}", f),
        }
    }

    pub fn recv_frame_headers_check(&mut self, stream_id: StreamId, end: bool) -> Headers {
        let headers = self.recv_frame_headers();
        assert_eq!(stream_id, headers.stream_id);
        assert_eq!(end, headers.is_end_of_stream());
        Headers(self.conn.decoder.decode(headers.header_fragment()).expect("decode"))
    }

    pub fn recv_frame_data_check(&mut self, stream_id: StreamId, end: bool) -> Vec<u8> {
        let data = self.recv_frame_data();
        assert_eq!(stream_id, data.stream_id);
        assert_eq!(end, data.is_end_of_stream());
        (&data.data[..]).to_vec()
    }
}
