use std::io;
use std::io::Read;

use futures::Future;
use futures::failed;

use futures::stream::Stream;

use futures_io::read_exact;
use futures_io::write_all;
use futures_mio::TcpStream;

use solicit::http::frame::RawFrame;
use solicit::http::connection::HttpFrame;
use solicit::http::frame::unpack_header;

use result::GrpcError;

use futures_grpc::GrpcFuture;
use futures_grpc::GrpcStream;

use io_misc::*;


struct VecWithPos<T> {
    vec: Vec<T>,
    pos: usize,
}

impl<T> AsMut<[T]> for VecWithPos<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.vec[self.pos..]
    }
}

fn recv_raw_frame<R : Read + Send + 'static>(read: R) -> GrpcFuture<(R, RawFrame<'static>)> {
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
        .map_err(|e| GrpcError::Other)
        .boxed()
}

pub fn initial(conn: TcpStream) -> GrpcFuture<TcpStream> {
    let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    let write_preface = write_all(conn, preface).map_err(|_| GrpcError::Other);

    let settings = write_preface.and_then(|(conn, _)| {
        recv_raw_frame(conn)
    });

    let done = settings.map(|(conn, settings_raw)| {
        match HttpFrame::from_raw(&settings_raw).unwrap() {
            HttpFrame::SettingsFrame(..) => (),
            _ => panic!("expecting settings frame"),
        };

        conn
    });

    done.boxed()
}
