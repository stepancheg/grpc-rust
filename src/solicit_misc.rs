use std::fmt;
use std::io;
use std::mem;
use std::cmp;
use std::str;

use solicit::http::Header;
use solicit::http::StreamId;
use solicit::http::StaticHeader;
use solicit::http::HttpResult;
use solicit::http::frame::*;
use solicit::http::connection::ReceiveFrame;
use solicit::http::connection::HttpFrame;
use solicit::http::connection::SendFrame;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::DataChunk;

use misc::*;
use http_common::*;


#[allow(dead_code)]
pub struct HeaderDebug<'a>(pub &'a Header<'a, 'a>);

impl<'a> fmt::Debug for HeaderDebug<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
	    write!(fmt, "Header {{ name: {:?}, value: {:?} }}",
	    	BsDebug(self.0.name()), BsDebug(self.0.value()))
	}
}



/// work around https://github.com/mlalic/solicit/pull/33
#[allow(dead_code)]
pub struct OnceReceiveFrame<'a> {
    raw_frame: RawFrame<'a>,
    used: bool,
}

#[allow(dead_code)]
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
        frame.serialize_into(&mut cursor)?;
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
        headers: &[StaticHeader],
        end_stream: EndStream)
            -> HttpResult<()>
    {
        self.conn().sender(send).send_headers(headers, stream_id, end_stream)
    }

    fn send_headers_to_vec(
        &mut self,
        stream_id: StreamId,
        headers: &[StaticHeader],
        end_stream: EndStream)
             -> HttpResult<Vec<u8>>
    {
        let mut send = VecSendFrame(Vec::new());
        self.send_headers(&mut send, stream_id, headers, end_stream)?;
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

            self.send_data_frame(send, stream_id, &data[pos..end], end_stream)?;

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
        self.send_data_frames(&mut send, stream_id, data, end_stream)?;
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
        self.send_end_of_stream(&mut send, stream_id)?;
        Ok(send.0)
    }

    fn send_part<S : SendFrame>(
        &mut self,
        send: &mut S,
        stream_id: StreamId,
        part: &HttpStreamPart)
            -> HttpResult<()>
    {
        let end_stream = if part.last { EndStream::Yes } else { EndStream::No };
        match part.content {
            HttpStreamPartContent::Data(ref data) => self.send_data_frames(send, stream_id, &data, end_stream),
            HttpStreamPartContent::Headers(ref headers) => self.send_headers(send, stream_id, &headers, end_stream),
        }
    }

    fn send_part_to_vec(
        &mut self,
        stream_id: StreamId,
        part: &HttpStreamPart)
            -> HttpResult<Vec<u8>>
    {
        let mut send = VecSendFrame(Vec::new());
        self.send_part(&mut send, stream_id, part)?;
        Ok(send.0)
    }
}

impl HttpConnectionEx for HttpConnection {
    fn conn(&mut self) -> &mut HttpConnection {
        self
    }
}

/// Frames with stream
#[derive(Debug)]
pub enum HttpFrameStream<'a> {
    Data(DataFrame<'a>),
    Headers(HeadersFrame<'a>),
    RstStream(RstStreamFrame),
    WindowUpdate(WindowUpdateFrame),
}

impl<'a> HttpFrameStream<'a> {
    #[allow(dead_code)]
    pub fn into_frame(self) -> HttpFrame<'a> {
        match self {
            HttpFrameStream::WindowUpdate(f) => HttpFrame::WindowUpdateFrame(f),
            HttpFrameStream::Data(f) => HttpFrame::DataFrame(f),
            HttpFrameStream::Headers(f) => HttpFrame::HeadersFrame(f),
            HttpFrameStream::RstStream(f) => HttpFrame::RstStreamFrame(f),
        }
    }

    #[allow(dead_code)]
    pub fn get_stream_id(&self) -> StreamId {
        match self {
            &HttpFrameStream::WindowUpdate(ref f) => f.get_stream_id(),
            &HttpFrameStream::Data(ref f) => f.get_stream_id(),
            &HttpFrameStream::Headers(ref f) => f.get_stream_id(),
            &HttpFrameStream::RstStream(ref f) => f.get_stream_id(),
        }
    }

    #[allow(dead_code)]
    pub fn is_end_of_stream(&self) -> bool {
        match self {
            &HttpFrameStream::WindowUpdate(..) => false,
            &HttpFrameStream::Data(ref f) => f.is_end_of_stream(),
            &HttpFrameStream::Headers(ref f) => f.is_end_of_stream(),
            &HttpFrameStream::RstStream(..) => true,
        }
    }

}

/// Frames without stream (zero stream id)
#[derive(Debug)]
pub enum HttpFrameConn<'a> {
    Settings(SettingsFrame),
    Ping(PingFrame),
    Goaway(GoawayFrame<'a>),
    WindowUpdate(WindowUpdateFrame),
}

impl<'a> HttpFrameConn<'a> {
    #[allow(dead_code)]
    pub fn into_frame(self) -> HttpFrame<'a> {
        match self {
            HttpFrameConn::Settings(f) => HttpFrame::SettingsFrame(f),
            HttpFrameConn::Ping(f) => HttpFrame::PingFrame(f),
            HttpFrameConn::Goaway(f) => HttpFrame::GoawayFrame(f),
            HttpFrameConn::WindowUpdate(f) => HttpFrame::WindowUpdateFrame(f),
        }
    }
}

#[derive(Debug)]
pub enum HttpFrameClassified<'a> {
    Stream(HttpFrameStream<'a>),
    Conn(HttpFrameConn<'a>),
    Unknown(RawFrame<'a>),
}

impl<'a> HttpFrameClassified<'a> {
    pub fn from(frame: HttpFrame<'a>) -> Self {
        match frame {
            HttpFrame::DataFrame(f) => HttpFrameClassified::Stream(HttpFrameStream::Data(f)),
            HttpFrame::HeadersFrame(f) => HttpFrameClassified::Stream(HttpFrameStream::Headers(f)),
            HttpFrame::RstStreamFrame(f) => HttpFrameClassified::Stream(HttpFrameStream::RstStream(f)),
            HttpFrame::SettingsFrame(f) => HttpFrameClassified::Conn(HttpFrameConn::Settings(f)),
            HttpFrame::PingFrame(f) => HttpFrameClassified::Conn(HttpFrameConn::Ping(f)),
            HttpFrame::GoawayFrame(f) => HttpFrameClassified::Conn(HttpFrameConn::Goaway(f)),
            HttpFrame::WindowUpdateFrame(f) => {
                if f.get_stream_id() != 0 {
                    HttpFrameClassified::Stream(HttpFrameStream::WindowUpdate(f))
                } else {
                    HttpFrameClassified::Conn(HttpFrameConn::WindowUpdate(f))
                }
            },
            HttpFrame::UnknownFrame(f) => HttpFrameClassified::Unknown(f),
        }
    }

    pub fn from_raw(raw_frame: &'a RawFrame) -> HttpResult<HttpFrameClassified<'a>> {
        Ok(HttpFrameClassified::from(HttpFrame::from_raw(raw_frame)?))
    }
}


pub fn slice_get_header<'a>(headers: &'a [Header<'a, 'a>], name: &str) -> Option<&'a str> {
    headers.iter()
        .find(|h| h.name() == name.as_bytes())
        .and_then(|h| str::from_utf8(h.value()).ok())
}

