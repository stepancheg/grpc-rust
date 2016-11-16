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
use solicit::http::connection::HttpConnection;

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
    fn new_data_chunk(&mut self, data: &[u8], last: bool);
    fn close_remote(&mut self);
}


pub trait HttpReadLoopInner : 'static {
    type LoopHttpStream : GrpcHttpStream;

    fn conn(&mut self) -> &mut HttpConnection;
    fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut Self::LoopHttpStream>;
    fn remove_stream(&mut self, stream_id: StreamId);

    /// Send a frame back to the network
    /// Must not be data frame
    fn send_frame<R : FrameIR>(&mut self, frame: R);
    fn out_window_increased(&mut self, stream_id: Option<StreamId>);

    fn process_headers_frame(&mut self, frame: HeadersFrame);

    fn process_settings_global(&mut self, _frame: SettingsFrame) {
        // TODO: apply settings
        // TODO: send ack
    }

    fn process_stream_window_update_frame(&mut self, frame: WindowUpdateFrame) {
        {
            let stream = self.get_stream_mut(frame.get_stream_id()).expect("stream not found");
            stream.common_mut().out_window_size.try_increase(frame.increment())
                .expect("failed to increment stream window");
        }
        self.out_window_increased(Some(frame.get_stream_id()));
    }

    fn process_conn_window_update(&mut self, frame: WindowUpdateFrame) {
        self.conn().out_window_size.try_increase(frame.increment())
            .expect("failed to increment conn window");
        self.out_window_increased(None);
    }

    fn process_rst_stream_frame(&mut self, _frame: RstStreamFrame) {
    }

    fn process_data_frame(&mut self, frame: DataFrame) {
        let stream_id = frame.get_stream_id();

        self.conn().decrease_in_window(frame.payload_len())
            .expect("failed to decrease conn win");

        let increment_conn =
            if self.conn().in_window_size() < INITIAL_CONNECTION_WINDOW_SIZE / 2 {
                let increment = INITIAL_CONNECTION_WINDOW_SIZE as u32;
                self.conn().in_window_size.try_increase(increment).expect("failed to increase");

                Some(increment)
            } else {
                None
            };

        let increment_stream = {
            let stream = self.get_stream_mut(frame.get_stream_id()).expect("stream not found");

            stream.common_mut().in_window_size.try_decrease(frame.payload_len() as i32)
                .expect("failed to decrease stream win");

            let increment_stream =
                if stream.common_mut().in_window_size.size() < INITIAL_CONNECTION_WINDOW_SIZE / 2 {
                    let increment = INITIAL_CONNECTION_WINDOW_SIZE as u32;
                    stream.common_mut().in_window_size.try_increase(increment).expect("failed to increase");

                    Some(increment)
                } else {
                    None
                };

            stream.new_data_chunk(&frame.data.as_ref(), frame.is_end_of_stream());

            if frame.is_end_of_stream() {
                stream.close_remote();
                // TODO: GC streams
            }

            // TODO: drop stream if closed on both ends

            increment_stream
        };

        if let Some(increment_conn) = increment_conn {
            self.send_frame(WindowUpdateFrame::for_connection(increment_conn));
        }

        if let Some(increment_stream) = increment_stream {
            self.send_frame(WindowUpdateFrame::for_stream(stream_id, increment_stream));
        }
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
            HttpFrameStream::WindowUpdate(window_update) => self.process_stream_window_update_frame(window_update),
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

    fn close_remote(&mut self, stream_id: StreamId) {
        debug!("close remote: {}", stream_id);

        let remove = {
            let mut stream = self.get_stream_mut(stream_id).expect("stream not found");
            stream.close_remote();
            stream.common().state == StreamState::Closed
        };
        if remove {
            self.remove_stream(stream_id);
        }
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
