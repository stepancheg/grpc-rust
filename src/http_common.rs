use futures::stream::Stream;
use futures::Future;
use futures;

use tokio_core::io::ReadHalf;
use tokio_core::io::Io;

use solicit::http::session::StreamState;
use solicit::http::frame::*;
use solicit::http::StaticHeader;
use solicit::http::StreamId;
use solicit::http::HttpError;
use solicit::http::WindowSize;
use solicit::http::INITIAL_CONNECTION_WINDOW_SIZE;

use futures_misc::*;

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


pub struct GrpcHttpStreamCommon {
    pub state: StreamState,
    pub out_window_size: WindowSize,
    pub in_window_size: WindowSize,
}

impl GrpcHttpStreamCommon {
    pub fn new() -> GrpcHttpStreamCommon {
        GrpcHttpStreamCommon {
            state: StreamState::Open,
            in_window_size: WindowSize::new(INITIAL_CONNECTION_WINDOW_SIZE),
            out_window_size: WindowSize::new(INITIAL_CONNECTION_WINDOW_SIZE),
        }
    }
}


pub trait GrpcHttpStream {
    fn common(&self) -> &GrpcHttpStreamCommon;
    fn common_mut(&mut self) -> &mut GrpcHttpStreamCommon;
}


pub trait HttpReadLoopInner : 'static {
    /// Send a frame back to the network
    fn send_frame<R : FrameIR>(&mut self, frame: R);
    fn process_headers_frame(&mut self, frame: HeadersFrame);
    fn process_data_frame(&mut self, frame: DataFrame);
    fn process_window_update_frame(&mut self, frame: WindowUpdateFrame);
    fn process_settings_global(&mut self, frame: SettingsFrame);
    fn process_conn_window_update(&mut self, frame: WindowUpdateFrame);
    fn process_rst_stream_frame(&mut self, _frame: RstStreamFrame) {
        // TODO
    }

    fn process_ping(&mut self, frame: PingFrame) {
        if frame.is_ack() {

        } else {
            self.send_frame(PingFrame::new_ack(frame.opaque_data()));
        }
    }

    fn process_conn_frame(&mut self, frame: HttpFrameConn) {
        match frame {
            HttpFrameConn::Settings(f) => self.process_settings_global(f),
            HttpFrameConn::Ping(f) => self.process_ping(f),
            HttpFrameConn::Goaway(_f) => panic!("TODO"),
            HttpFrameConn::WindowUpdate(f) => self.process_conn_window_update(f),
        }
    }

    fn process_stream_frame(&mut self, frame: HttpFrameStream) {
        let stream_id = frame.get_stream_id();
        let end_of_stream = frame.is_end_of_stream();
        match frame {
            HttpFrameStream::Data(data) => self.process_data_frame(data),
            HttpFrameStream::Headers(headers) => self.process_headers_frame(headers),
            HttpFrameStream::RstStream(rst) => self.process_rst_stream_frame(rst),
            HttpFrameStream::WindowUpdate(window_update) => self.process_window_update_frame(window_update),
        };
        if end_of_stream {
            self.close_remote(stream_id);
        }
    }

    fn process_raw_frame(&mut self, raw_frame: RawFrame) {
        let frame = HttpFrameClassified::from_raw(&raw_frame).unwrap();
        debug!("received frame: {:?}", frame);
        match frame {
            HttpFrameClassified::Conn(f) => self.process_conn_frame(f),
            HttpFrameClassified::Stream(f) => self.process_stream_frame(f),
            HttpFrameClassified::Unknown(_f) => {},
        }
    }

    fn close_remote(&mut self, _stream_id: StreamId) {
        // TODO
    }
}


pub struct HttpReadLoopData<I, N>
    where
        I : Io + 'static,
        N : HttpReadLoopInner,
{
    pub read: ReadHalf<I>,
    pub inner: TaskRcMut<N>,
}

impl<I, N> HttpReadLoopData<I, N>
    where
        I : Io + Send + 'static,
        N : HttpReadLoopInner,
{
    /// Recv a frame from the network
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let HttpReadLoopData { read, inner } = self;
        Box::new(recv_raw_frame(read)
            .map(|(read, frame)| (HttpReadLoopData { read: read, inner: inner }, frame))
            .map_err(HttpError::from))
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        Box::new(self.recv_raw_frame()
            .and_then(move |(lp, frame)| lp.process_raw_frame(frame)))
    }

    pub fn run(self) -> HttpFuture<()> {
        let stream = stream_repeat(());

        let future = stream.fold(self, |lp, _| {
            lp.read_process_frame()
        });

        Box::new(future.map(|_| ()))
    }

    fn process_raw_frame(self, frame: RawFrame) -> HttpFuture<Self> {
        self.inner.with(move |inner| {
            inner.process_raw_frame(frame);
        });
        Box::new(futures::finished(self))
    }
}
