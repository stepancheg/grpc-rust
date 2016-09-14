use futures::stream::Stream;
use futures;

use solicit::http::StaticHeader;
use solicit::http::HttpError;
use solicit::http::frame::*;

use solicit_misc::*;
use solicit_async::*;


#[derive(Debug)]
pub enum HttpStreamPartContent {
    Headers(Vec<StaticHeader>),
    Data(Vec<u8>),
}

pub struct HttpStreamPart {
    pub content: HttpStreamPartContent,
    /// END_STREAM
    pub last: bool,
}

impl HttpStreamPart {
    pub fn last_headers(header: Vec<StaticHeader>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Headers(header),
            last: true,
        }
    }

    pub fn intermediate_headers(headers: Vec<StaticHeader>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Headers(headers),
            last: false,
        }
    }

    pub fn intermediate_data(data: Vec<u8>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Data(data),
            last: false,
        }
    }

    pub fn last_data(data: Vec<u8>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Data(data),
            last: true,
        }
    }
}

pub type HttpStreamStream = Box<Stream<Item=HttpStreamPart, Error=HttpError>>;
pub type HttpStreamStreamSend = Box<Stream<Item=HttpStreamPart, Error=HttpError> + Send>;


pub trait HttpService: Send + 'static {
    fn new_request(&mut self, headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend;
}


pub trait HttpReadLoop
    where Self : Sized + 'static
{
    fn process_data_frame(self, frame: DataFrame) -> HttpFuture<Self>;
    fn process_headers_frame(self, frame: HeadersFrame) -> HttpFuture<Self>;
    fn process_rst_stream_frame(self, frame: RstStreamFrame) -> HttpFuture<Self>;
    fn process_window_update_frame(self, _frame: WindowUpdateFrame) -> HttpFuture<Self>;
    fn process_settings_global(self, _frame: SettingsFrame) -> HttpFuture<Self>;
    fn process_conn_window_update(self, _frame: WindowUpdateFrame) -> HttpFuture<Self>;

    /// Send a frame back to the network
    fn send_frame<R : FrameIR>(self, frame: R) -> HttpFuture<Self>;

    fn process_stream_frame(self, frame: HttpFrameStream) -> HttpFuture<Self> {
        match frame {
            HttpFrameStream::Data(data) => self.process_data_frame(data),
            HttpFrameStream::Headers(headers) => self.process_headers_frame(headers),
            HttpFrameStream::RstStream(rst) => self.process_rst_stream_frame(rst),
            HttpFrameStream::WindowUpdate(window_update) => self.process_window_update_frame(window_update),
        }
    }

    fn process_ping(self, frame: PingFrame) -> HttpFuture<Self> {
        if frame.is_ack() {
            Box::new(futures::finished(self))
        } else {
            self.send_frame(PingFrame::new_ack(frame.opaque_data()))
        }
    }

    fn process_conn_frame(self, frame: HttpFrameConn) -> HttpFuture<Self> {
        match frame {
            HttpFrameConn::Settings(f) => self.process_settings_global(f),
            HttpFrameConn::Ping(f) => self.process_ping(f),
            HttpFrameConn::Goaway(_f) => panic!("TODO"),
            HttpFrameConn::WindowUpdate(f) => self.process_conn_window_update(f),
        }
    }

    fn process_raw_frame(self, raw_frame: RawFrame) -> HttpFuture<Self> {
        let frame = HttpFrameClassified::from_raw(&raw_frame).unwrap();
        match frame {
            HttpFrameClassified::Conn(f) => self.process_conn_frame(f),
            HttpFrameClassified::Stream(f) => self.process_stream_frame(f),
            HttpFrameClassified::Unknown(_f) => Box::new(futures::finished(self)),
        }
    }
}
