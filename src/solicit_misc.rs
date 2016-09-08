use std::fmt;
use std::io;
use std::mem;
use std::cmp;

use solicit::http::Header;
use solicit::http::StreamId;
use solicit::http::StaticHeader;
use solicit::http::HttpResult;
use solicit::http::frame::RawFrame;
use solicit::http::frame::FrameIR;
use solicit::http::connection::ReceiveFrame;
use solicit::http::connection::HttpFrame;
use solicit::http::connection::SendFrame;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::DataChunk;

use misc::*;


#[allow(dead_code)]
pub struct HeaderDebug<'a>(pub &'a Header<'a, 'a>);

impl<'a> fmt::Debug for HeaderDebug<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
	    write!(fmt, "Header {{ name: {:?}, value: {:?} }}",
	    	BsDebug(self.0.name()), BsDebug(self.0.value()))
	}
}



/// work around https://github.com/mlalic/solicit/pull/33
pub struct OnceReceiveFrame<'a> {
    raw_frame: RawFrame<'a>,
    used: bool,
}

impl<'a> OnceReceiveFrame<'a> {
    pub fn new(raw_frame: RawFrame<'a>) -> OnceReceiveFrame<'a> {
        OnceReceiveFrame {
            raw_frame: raw_frame,
            used: false,
        }
    }
}

impl<'a> ReceiveFrame for OnceReceiveFrame<'a> {
    fn recv_frame(&mut self) -> HttpResult<HttpFrame> {
        assert!(!self.used);
        self.used = true;
        HttpFrame::from_raw(&self.raw_frame)
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

pub trait HttpConnectionEx {
    fn conn(&mut self) -> &mut HttpConnection;

    fn send_headers<S : SendFrame>(
        &mut self,
        send: &mut S,
        stream_id: StreamId,
        headers: Vec<StaticHeader>,
        end_stream: EndStream)
            -> HttpResult<()>
    {
        self.conn().sender(send).send_headers(headers, stream_id, EndStream::No)
    }

    fn send_headers_to_vec(
        &mut self,
        stream_id: StreamId,
        headers: Vec<StaticHeader>,
        end_stream: EndStream)
             -> HttpResult<Vec<u8>>
    {
        let mut send = VecSendFrame(Vec::new());
        try!(self.send_headers(&mut send, stream_id, headers, end_stream));
        Ok(send.0)
    }

    fn send_data_frame<S : SendFrame>(
        &mut self,
        send: &mut S,
        stream_id: StreamId,
        chunk: &[u8],
        end_stream: EndStream)
            -> HttpResult<()>
    {
        let data_chunk = DataChunk::new_borrowed(chunk, stream_id, end_stream);
        self.conn().sender(send).send_data(data_chunk)
    }

    fn send_data_frames<S : SendFrame>(
        &mut self,
        send: &mut S,
        stream_id: StreamId,
        data: &[u8],
        end_stream: EndStream)
            -> HttpResult<()>
    {
        // if client requested end of stream,
        // we must send at least one frame with end stream flag
        if end_stream == EndStream::Yes && data.len() == 0 {
            return self.send_end_of_stream(send, stream_id)
        }

        let mut pos = 0;
        const MAX_CHUNK_SIZE: usize = 8 * 1024;
        while pos < data.len() {
            let end = cmp::min(data.len(), pos + MAX_CHUNK_SIZE);

            let end_stream =
                if end == data.len() && end_stream == EndStream::Yes {
                    EndStream::Yes
                } else {
                    EndStream::No
                };

            try!(self.send_data_frame(send, stream_id, &data[pos..end], end_stream));

            pos = end;
        }

        Ok(())
    }

    fn send_data_frames_to_vec(
        &mut self,
        stream_id: StreamId,
        data: &[u8],
        end_stream: EndStream)
            -> HttpResult<Vec<u8>>
    {
        let mut send = VecSendFrame(Vec::new());
        try!(self.send_data_frames(&mut send, stream_id, data, end_stream));
        Ok(send.0)
    }

    fn send_end_of_stream<S : SendFrame>(
        &mut self,
        send: &mut S,
        stream_id: StreamId)
            -> HttpResult<()>
    {
        self.send_data_frame(send, stream_id, &Vec::new(), EndStream::Yes)
    }

    fn send_end_of_stream_to_vec(&mut self, stream_id: StreamId) -> HttpResult<Vec<u8>> {
        let mut send = VecSendFrame(Vec::new());
        try!(self.send_end_of_stream(&mut send, stream_id));
        Ok(send.0)
    }
}

impl HttpConnectionEx for HttpConnection {
    fn conn(&mut self) -> &mut HttpConnection {
        self
    }
}
