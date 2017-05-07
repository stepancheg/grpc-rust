use std::io::Read;
use std::net::SocketAddr;

use futures::done;
use futures::future::Future;
use futures::future::BoxFuture;
use futures::stream::Stream;
use futures::stream::BoxStream;

use tokio_io::io::read_exact;
use tokio_io::io::write_all;
use tokio_core::net::TcpStream;
use tokio_core::reactor;
use tokio_io::AsyncWrite;
use tokio_io::AsyncRead;

use solicit::HttpError;
use solicit::HttpResult;
use solicit::frame::FRAME_HEADER_LEN;
use solicit::frame::RawFrame;
use solicit::frame::RawFrameRef;
use solicit::frame::FrameIR;
use solicit::frame::unpack_header;
use solicit::frame::settings::SettingsFrame;
use solicit::frame::settings::HttpSetting;
use solicit::connection::HttpFrame;


pub type HttpFuture<T> = Box<Future<Item=T, Error=HttpError>>;
// Type is called `HttpFutureStream`, not just `HttpStream`
// to avoid confusion with streams from HTTP/2 spec
pub type HttpFutureStream<T> = Box<Stream<Item=T, Error=HttpError>>;

pub type HttpFutureSend<T> = BoxFuture<T, HttpError>;
pub type HttpFutureStreamSend<T> = BoxStream<T, HttpError>;


struct VecWithPos<T> {
    vec: Vec<T>,
    pos: usize,
}

impl<T> AsMut<[T]> for VecWithPos<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.vec[self.pos..]
    }
}

pub fn recv_raw_frame<R : AsyncRead + Send + 'static>(read: R) -> HttpFuture<(R, RawFrame)> {
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

pub fn recv_settings_frame<R : AsyncRead + Send + 'static>(read: R) -> HttpFuture<(R, SettingsFrame)> {
    Box::new(recv_raw_frame(read)
        .then(|result| {
            result.and_then(|(read, raw_frame)| {
                match HttpFrame::from_raw(&raw_frame) {
                    Ok(HttpFrame::SettingsFrame(f)) => Ok((read, f)),
                    Ok(f) => Err(HttpError::InvalidFrame(format!("unexpected frame, expected SETTINGS, got {:?}", f.frame_type()))),
                    Err(e) => Err(e),
                }
            })
        }))
}

pub fn recv_settings_frame_ack<R : AsyncRead + Send + 'static>(read: R) -> HttpFuture<(R, SettingsFrame)> {
    Box::new(recv_settings_frame(read).and_then(|(read, frame)| {
        if frame.is_ack() {
            Ok((read, frame))
        } else {
            Err(HttpError::InvalidFrame("expecting SETTINGS with ack, got without ack".to_owned()))
        }
    }))
}

pub fn recv_settings_frame_set<R : AsyncRead + Send + 'static>(read: R) -> HttpFuture<(R, SettingsFrame)> {
    Box::new(recv_settings_frame(read).and_then(|(read, frame)| {
        if !frame.is_ack() {
            Ok((read, frame))
        } else {
            Err(HttpError::InvalidFrame("expecting SETTINGS without ack, got with ack".to_owned()))
        }
    }))
}

#[allow(dead_code)]
pub fn send_raw_frame<W : AsyncWrite + Send + 'static>(write: W, frame: RawFrame) -> HttpFuture<W> {
    let bytes = frame.serialize();
    Box::new(write_all(write, bytes.clone())
        .map(|(w, _)| w)
        .map_err(|e| e.into()))
}

pub fn send_frame<W : AsyncWrite + Send + 'static, F : FrameIR>(write: W, frame: F) -> HttpFuture<W> {
    let buf = frame.serialize_into_vec();
    debug!("send frame {}", RawFrameRef { raw_content: &buf }.frame_type());
    Box::new(write_all(write, buf)
        .map(|(w, _)| w)
        .map_err(|e| e.into()))
}

static PREFACE: &'static [u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

fn send_settings<W : AsyncWrite + Send + 'static>(conn: W) -> HttpFuture<W> {
    let settings = {
        let mut frame = SettingsFrame::new();
        frame.add_setting(HttpSetting::EnablePush(0));
        frame
    };

    Box::new(send_frame(conn, settings))
}

pub fn client_handshake<I : AsyncWrite + AsyncRead + Send + 'static>(conn: I) -> HttpFuture<I> {
    debug!("send PREFACE");
    let send_preface = write_all(conn, PREFACE)
        .map(|(conn, _)| conn)
        .map_err(|e| e.into());

    let send_settings = send_preface.and_then(send_settings);

    Box::new(send_settings)
}

pub fn server_handshake<I : AsyncRead + AsyncWrite + Send + 'static>(conn: I) -> HttpFuture<I> {
    let mut preface_buf = Vec::with_capacity(PREFACE.len());
    preface_buf.resize(PREFACE.len(), 0);
    let recv_preface = read_exact(conn, preface_buf)
        .map_err(|e| e.into())
        .and_then(|(conn, preface_buf)| {
            done(if preface_buf == PREFACE {
                Ok((conn))
            } else {
                Err(HttpError::InvalidFrame("wrong preface".to_owned()))
            })
        });

    let send_settings = recv_preface.and_then(send_settings);

    Box::new(send_settings)
}

pub fn connect_and_handshake(lh: &reactor::Handle, addr: &SocketAddr) -> HttpFuture<TcpStream> {
    let connect = TcpStream::connect(&addr, lh)
        .map_err(|e| e.into());

    let handshake = connect.and_then(client_handshake);

    Box::new(handshake)
}
