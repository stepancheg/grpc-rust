use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;

use futures::done;
use futures::Future;
use futures::stream::Stream;

use tokio_core::io::read_exact;
use tokio_core::io::write_all;
use tokio_core::io::Io;
use tokio_core::net::TcpStream;
use tokio_core::reactor;

use solicit::HttpError;
use solicit::HttpResult;
use solicit::frame::FRAME_HEADER_LEN;
use solicit::frame::RawFrame;
use solicit::frame::FrameIR;
use solicit::frame::unpack_header;
use solicit::frame::settings::SettingsFrame;
use solicit::frame::settings::HttpSetting;
use solicit::connection::HttpFrame;


pub type HttpFuture<T> = Box<Future<Item=T, Error=HttpError>>;
// Type is called `HttpFutureStream`, not just `HttpStream`
// to avoid confusion with streams from HTTP/2 spec
pub type HttpFutureStream<T> = Box<Stream<Item=T, Error=HttpError>>;

pub type HttpFutureSend<T> = Box<Future<Item=T, Error=HttpError> + Send>;
pub type HttpFutureStreamSend<T> = Box<Stream<Item=T, Error=HttpError> + Send>;


struct VecWithPos<T> {
    vec: Vec<T>,
    pos: usize,
}

impl<T> AsMut<[T]> for VecWithPos<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.vec[self.pos..]
    }
}

pub fn recv_raw_frame<R : Read + Send + 'static>(read: R) -> HttpFuture<(R, RawFrame)> {
    let header = read_exact(read, [0; FRAME_HEADER_LEN]);
    let frame_buf = header.and_then(|(read, raw_header)| {
        let header = unpack_header(&raw_header);
        let total_len = FRAME_HEADER_LEN + header.0 as usize;
        let mut full_frame = VecWithPos {
            vec: Vec::with_capacity(total_len),
            pos: 0,
        };

        full_frame.vec.extend(&raw_header);
        full_frame.vec.resize(total_len, 0);
        full_frame.pos = FRAME_HEADER_LEN;

        read_exact(read, full_frame)
    });
    let frame = frame_buf.map(|(read, frame_buf)| {
        (read, RawFrame::from(frame_buf.vec))
    });
    Box::new(frame
        .map_err(|e| e.into()))
}

pub fn recv_raw_frame_sync(read: &mut Read) -> HttpResult<RawFrame> {
    // TODO: copy-paste
    let mut raw_header = [0; FRAME_HEADER_LEN];
    read.read_exact(&mut raw_header)?;
    let header = unpack_header(&raw_header);

    let total_len = FRAME_HEADER_LEN + header.0 as usize;
    let mut full_frame = Vec::new();
    full_frame.extend_from_slice(&raw_header);
    full_frame.resize(total_len, 0);

    read.read_exact(&mut full_frame[FRAME_HEADER_LEN..])?;

    Ok(RawFrame::from(full_frame))
}

#[allow(dead_code)]
pub fn recv_raw_frame_stream<R : Read + Send + 'static>(_read: R) -> HttpFutureStream<RawFrame> {
    // https://users.rust-lang.org/t/futures-rs-how-to-generate-a-stream-from-futures/7020
    panic!();
}

pub fn recv_settings_frame<R : Read + Send + 'static>(read: R) -> HttpFuture<(R, SettingsFrame)> {
    Box::new(recv_raw_frame(read)
        .then(|result| {
            result.and_then(|(read, raw_frame)| {
                match HttpFrame::from_raw(&raw_frame) {
                    Ok(HttpFrame::SettingsFrame(f)) => Ok((read, f)),
                    Ok(_) => Err(HttpError::InvalidFrame),
                    Err(e) => Err(e),
                }
            })
        }))
}

pub fn recv_settings_frame_ack<R : Read + Send + 'static>(read: R) -> HttpFuture<(R, SettingsFrame)> {
    Box::new(recv_settings_frame(read).and_then(|(read, frame)| {
        if frame.is_ack() {
            Ok((read, frame))
        } else {
            Err(HttpError::InvalidFrame)
        }
    }))
}

pub fn recv_settings_frame_set<R : Read + Send + 'static>(read: R) -> HttpFuture<(R, SettingsFrame)> {
    Box::new(recv_settings_frame(read).and_then(|(read, frame)| {
        if !frame.is_ack() {
            Ok((read, frame))
        } else {
            Err(HttpError::InvalidFrame)
        }
    }))
}

#[allow(dead_code)]
pub fn send_raw_frame<W : Write + Send + 'static>(write: W, frame: RawFrame) -> HttpFuture<W> {
    let bytes = frame.serialize();
    Box::new(write_all(write, bytes.clone())
        .map(|(w, _)| w)
        .map_err(|e| e.into()))
}

pub fn send_frame<W : Write + Send + 'static, F : FrameIR>(write: W, frame: F) -> HttpFuture<W> {
    let buf = frame.serialize_into_vec();
    Box::new(write_all(write, buf)
        .map(|(w, _)| w)
        .map_err(|e| e.into()))
}

static PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Send and receive settings frames and acks
fn settings_xchg<I : Io + Send + 'static>(conn: I) -> HttpFuture<I> {
    let settings = {
        let mut frame = SettingsFrame::new();
        frame.add_setting(HttpSetting::EnablePush(0));
        frame
    };

    let send_settings = send_frame(conn, settings);

    let recv_settings = send_settings.and_then(|conn| {
        recv_settings_frame_set(conn).map(|(conn, _)| conn)
    });

    let send_settings_ack = recv_settings.and_then(|conn| {
        send_frame(conn, SettingsFrame::new_ack())
    });

    let recv_settings_ack = send_settings_ack.and_then(|conn| {
        recv_settings_frame_ack(conn).map(|(conn, _)| conn)
    });

    Box::new(recv_settings_ack)
}

pub fn client_handshake<I : Io + Send + 'static>(conn: I) -> HttpFuture<I> {
    let send_preface = write_all(conn, PREFACE)
        .map(|(conn, _)| conn)
        .map_err(|e| e.into());

    let settings_xchg = send_preface.and_then(settings_xchg);

    Box::new(settings_xchg)
}

pub fn server_handshake<I : Io + Send + 'static>(conn: I) -> HttpFuture<I> {
    let mut preface_buf = Vec::with_capacity(PREFACE.len());
    preface_buf.resize(PREFACE.len(), 0);
    let recv_preface = read_exact(conn, preface_buf)
        .map_err(|e| e.into())
        .and_then(|(conn, preface_buf)| {
            done(if preface_buf == PREFACE {
                Ok((conn))
            } else {
                Err(HttpError::InvalidFrame)
            })
        });

    let settings_xchg = recv_preface.and_then(settings_xchg);

    Box::new(settings_xchg)
}

pub fn connect_and_handshake(lh: &reactor::Handle, addr: &SocketAddr) -> HttpFuture<TcpStream> {
    let connect = TcpStream::connect(&addr, lh)
        .map_err(|e| e.into());

    let handshake = connect.and_then(client_handshake);

    Box::new(handshake)
}
