use std::io;
use std::io::Read;
use std::io::Write;
use std::mem;

use futures::Future;
use futures::BoxFuture;
use futures::failed;

use futures::stream::Stream;
use futures::stream::BoxStream;

use futures_io::read_exact;
use futures_io::write_all;
use futures_mio::TcpStream;

use solicit::http::HttpResult;
use solicit::http::HttpError;
use solicit::http::frame::RawFrame;
use solicit::http::frame::FrameIR;
use solicit::http::frame::unpack_header;
use solicit::http::frame::settings::SettingsFrame;
use solicit::http::frame::settings::HttpSetting;
use solicit::http::connection::HttpFrame;
use solicit::http::connection::ReceiveFrame;
use solicit::http::connection::SendFrame;
use solicit::http::transport::TransportStream;

use io_misc::*;


pub type HttpFuture<T> = BoxFuture<T, HttpError>;
pub type HttpStream<T> = BoxStream<T, HttpError>;


struct VecWithPos<T> {
    vec: Vec<T>,
    pos: usize,
}

impl<T> AsMut<[T]> for VecWithPos<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.vec[self.pos..]
    }
}

pub fn recv_raw_frame<R : Read + Send + 'static>(read: R) -> HttpFuture<(R, RawFrame<'static>)> {
    let header = read_exact(read, [0; 9]);
    let frame_buf = header.and_then(|(read, raw_header)| {
        let header = unpack_header(&raw_header);
        let total_len = 9 + header.0 as usize;
        let mut full_frame = VecWithPos {
            vec: Vec::with_capacity(9),
            pos: 0,
        };

        full_frame.vec.reserve_exact(total_len);
        full_frame.vec.extend(&raw_header);
        full_frame.vec.resize(total_len, 0);
        full_frame.pos = 9;

        read_exact(read, full_frame)
    });
    let frame = frame_buf.map(|(read, frame_buf)| {
        (read, RawFrame::from(frame_buf.vec))
    });
    frame
        .map_err(|e| e.into())
        .boxed()
}

pub fn send_raw_frame<W : Write + Send + 'static>(write: W, frame: RawFrame<'static>) -> HttpFuture<W> {
    let bytes = frame.serialize();
    write_all(write, bytes)
        .map(|(w, _)| w)
        .map_err(|e| e.into())
        .boxed()
}

pub fn send_frame<W : Write + Send + 'static, F : FrameIR>(write: W, frame: F) -> HttpFuture<W> {
    let mut buf = io::Cursor::new(Vec::with_capacity(16));
    frame.serialize_into(&mut buf).unwrap();
    write_all(write, buf.into_inner())
        .map(|(w, _)| w)
        .map_err(|e| e.into())
        .boxed()
}

pub fn initial(conn: TcpStream) -> HttpFuture<TcpStream> {
    let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    let write_preface = write_all(conn, preface)
        .map(|(conn, _)| conn)
        .map_err(|e| e.into());

    let write_settings = write_preface.and_then(|conn| {
        let settings = {
            let mut frame = SettingsFrame::new();
            frame.add_setting(HttpSetting::EnablePush(0));
            frame
        };
        send_frame(conn, settings)
    });

    let recv_settings = write_settings.and_then(|conn| {
        recv_raw_frame(conn).map(|(conn, settings_raw)| {
            match HttpFrame::from_raw(&settings_raw).unwrap() {
                HttpFrame::SettingsFrame(..) => (),
                _ => panic!("expecting settings frame"),
            };

            conn
        })
    });

    let done = recv_settings;

    done.boxed()
}


pub struct OnceReceiveFrame {
    raw_frame: RawFrame<'static>,
    used: bool,
}

impl OnceReceiveFrame {
    pub fn new(raw_frame: RawFrame<'static>) -> OnceReceiveFrame {
        OnceReceiveFrame {
            raw_frame: raw_frame,
            used: false,
        }
    }
}

impl ReceiveFrame for OnceReceiveFrame {
    fn recv_frame(&mut self) -> HttpResult<HttpFrame> {
        assert!(!self.used);
        self.used = true;
        let r = HttpFrame::from_raw(&self.raw_frame);
        if let Some(f) = r.as_ref().ok() {
        }
        r
    }
}

pub struct VecSendFrame(pub Vec<u8>);

impl SendFrame for VecSendFrame {
    fn send_frame<F : FrameIR>(&mut self, frame: F) -> HttpResult<()> {
        let pos = self.0.len();
        let mut cursor = io::Cursor::new(mem::replace(&mut self.0, Vec::new()));
        cursor.set_position(pos as u64);
        try!(frame.serialize_into(&mut cursor));
        self.0 = cursor.into_inner();

        Ok(())
    }
}
