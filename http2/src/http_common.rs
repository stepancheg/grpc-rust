use std::collections::HashMap;
use std::collections::VecDeque;
use std::cmp;

use futures::stream::Stream;
use futures::Future;
use futures::future;
use futures;

use bytes::Bytes;

use tokio_core::io::ReadHalf;
use tokio_core::io::WriteHalf;
use tokio_core::io::Io;
use tokio_core::io as tokio_io;

use solicit::session::StreamState;
use solicit::frame::*;
use solicit::Header;
use solicit::StreamId;
use solicit::HttpError;
use solicit::WindowSize;
use solicit::HttpScheme;
use solicit::INITIAL_CONNECTION_WINDOW_SIZE;
use solicit::connection::EndStream;
use solicit::connection::HttpConnection;
use solicit::connection::SendFrame;

use futures_misc::*;

use solicit_misc::*;
use solicit_async::*;


#[derive(Debug)]
pub enum HttpStreamPartContent {
    Headers(Vec<Header>),
    Data(Bytes),
}

pub struct HttpStreamPart {
    pub content: HttpStreamPartContent,
    /// END_STREAM
    pub last: bool,
}

impl HttpStreamPart {
    pub fn last_headers(header: Vec<Header>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Headers(header),
            last: true,
        }
    }

    pub fn intermediate_headers(headers: Vec<Header>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Headers(headers),
            last: false,
        }
    }

    pub fn intermediate_data(data: Vec<u8>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Data(Bytes::from(data)),
            last: false,
        }
    }

    pub fn last_data(data: Vec<u8>) -> Self {
        HttpStreamPart {
            content: HttpStreamPartContent::Data(Bytes::from(data)),
            last: true,
        }
    }
}

pub type HttpPartFutureStream = Box<Stream<Item=HttpStreamPart, Error=HttpError>>;
pub type HttpPartFutureStreamSend = Box<Stream<Item=HttpStreamPart, Error=HttpError> + Send>;


pub trait HttpService: Send + 'static {
    fn new_request(&self, headers: Vec<Header>, req: HttpPartFutureStreamSend) -> HttpPartFutureStreamSend;
}


pub enum CommonToWriteMessage {
    TryFlushStream(Option<StreamId>), // flush stream when window increased or new data added
    Write(Vec<u8>),
}


pub struct HttpStreamCommon {
    pub state: StreamState,
    pub out_window_size: WindowSize,
    pub in_window_size: WindowSize,
    pub outgoing: VecDeque<HttpStreamPartContent>,
    // Means nothing will be added to `outgoing`
    pub outgoing_end: bool,
}

impl HttpStreamCommon {
    pub fn new() -> HttpStreamCommon {
        HttpStreamCommon {
            state: StreamState::Open,
            in_window_size: WindowSize::new(INITIAL_CONNECTION_WINDOW_SIZE),
            out_window_size: WindowSize::new(INITIAL_CONNECTION_WINDOW_SIZE),
            outgoing: VecDeque::new(),
            outgoing_end: false,
        }
    }

    pub fn close_local(&mut self) {
        trace!("close local");
        self.state = match self.state {
            StreamState::Closed | StreamState::HalfClosedRemote => StreamState::Closed,
            _ => StreamState::HalfClosedLocal,
        };
    }

    pub fn close_remote(&mut self) {
        self.state = match self.state {
            StreamState::Closed | StreamState::HalfClosedLocal => StreamState::Closed,
            _ => StreamState::HalfClosedRemote,
        };
    }

    pub fn pop_outg(&mut self, conn_out_window_size: &mut WindowSize) -> Option<HttpStreamPart> {
        if self.outgoing.is_empty() {
            return
                if self.outgoing_end {
                    if self.state.is_closed_local() {
                        None
                    } else {
                        self.close_local();
                        Some(HttpStreamPart::last_data(Vec::new()))
                    }
                } else {
                    None
                };
        }

        let pop_headers =
            if let &HttpStreamPartContent::Headers(..) = self.outgoing.front().unwrap() {
                true
            } else {
                false
            };
        if pop_headers {
            let r = self.outgoing.pop_front().unwrap();
            let last = self.outgoing_end && self.outgoing.is_empty();
            if last {
                self.close_local();
            }
            return Some(HttpStreamPart {
                content: r,
                last: last,
            })
        }

        if self.out_window_size.size() <= 0 {
            return None
        }

        let mut data =
            if let Some(HttpStreamPartContent::Data(data)) = self.outgoing.pop_front() {
                data
            } else {
                unreachable!()
            };

        // Max of connection and stream window size
        let max_window = cmp::max(self.out_window_size.size(), conn_out_window_size.size());

        if data.len() as usize > max_window as usize {
            trace!("truncating data of len {} to {}", data.len(), max_window);
            let size = max_window as usize;
            let rem = data.split_off(size);
            self.outgoing.push_front(HttpStreamPartContent::Data(rem));
        };

        self.out_window_size.try_decrease(data.len() as i32).unwrap();
        conn_out_window_size.try_decrease(data.len() as i32).unwrap();

        let last = self.outgoing_end && self.outgoing.is_empty();
        if last {
            self.close_local();
        }

        Some(HttpStreamPart {
            content: HttpStreamPartContent::Data(data),
            last: last,
        })
    }

    pub fn pop_outg_all(&mut self, conn_out_window_size: &mut WindowSize) -> Vec<HttpStreamPart> {
        let mut r = Vec::new();
        while let Some(p) = self.pop_outg(conn_out_window_size) {
            r.push(p);
        }
        r
    }
}


pub trait HttpStream {
    fn common(&self) -> &HttpStreamCommon;
    fn common_mut(&mut self) -> &mut HttpStreamCommon;
    fn new_data_chunk(&mut self, data: &[u8], last: bool);
    fn closed_remote(&mut self);
}


pub struct LoopInnerCommon<S>
    where S : HttpStream,
{
    pub conn: HttpConnection,
    pub streams: HashMap<StreamId, S>,
}


#[derive(Debug)]
pub struct ConnectionStateSnapshot {
    pub streams: HashMap<StreamId, StreamState>,
}




impl<S> LoopInnerCommon<S>
    where S : HttpStream,
{
    pub fn new(scheme: HttpScheme) -> LoopInnerCommon<S> {
        LoopInnerCommon {
            conn: HttpConnection::new(scheme),
            streams: HashMap::new(),
        }
    }

    pub fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut S> {
        self.streams.get_mut(&stream_id)
    }

    pub fn remove_stream(&mut self, stream_id: StreamId) {
        match self.streams.remove(&stream_id) {
            Some(_) => debug!("removed stream: {}", stream_id),
            None => debug!("incorrect request to remove stream: {}", stream_id),
        }
    }

    pub fn remove_stream_if_closed(&mut self, stream_id: StreamId) {
        if self.get_stream_mut(stream_id).expect("unknown stream").common().state == StreamState::Closed {
            self.remove_stream(stream_id);
        }
    }


    pub fn pop_outg_for_stream(&mut self, stream_id: StreamId) -> Option<HttpStreamPart> {
        let r = {
            if let Some(stream) = self.streams.get_mut(&stream_id) {
                stream.common_mut().pop_outg(&mut self.conn.out_window_size)
            } else {
                None
            }
        };
        if let Some(..) = r {
            self.remove_stream_if_closed(stream_id);
        }
        r
    }

    pub fn pop_outg_for_conn(&mut self) -> Option<(StreamId, HttpStreamPart)> {
        // TODO: lame
        let stream_ids: Vec<StreamId> = self.streams.keys().cloned().collect();
        for stream_id in stream_ids {
            let r = self.pop_outg_for_stream(stream_id);
            if let Some(r) = r {
                return Some((stream_id, r));
            }
        }
        None
    }

    pub fn pop_outg_all_for_stream(&mut self, stream_id: StreamId) -> Vec<HttpStreamPart> {
        let mut r = Vec::new();
        while let Some(p) = self.pop_outg_for_stream(stream_id) {
            r.push(p);
        }
        r
    }

    pub fn pop_outg_all_for_conn(&mut self) -> Vec<(StreamId, HttpStreamPart)> {
        let mut r = Vec::new();
        while let Some(p) = self.pop_outg_for_conn() {
            r.push(p);
        }
        r
    }

    fn write_part(&mut self, target: &mut VecSendFrame, stream_id: StreamId, part: HttpStreamPart) {
        match part.content {
            HttpStreamPartContent::Data(data) => {
                // if client requested end of stream,
                // we must send at least one frame with end stream flag
                if part.last && data.len() == 0 {
                    let mut frame = DataFrame::with_data(stream_id, Vec::new());
                    frame.set_flag(DataFlag::EndStream);

                    debug!("sending frame {:?}", frame);

                    return target.send_frame(frame).unwrap();
                }

                let mut pos = 0;
                const MAX_CHUNK_SIZE: usize = 8 * 1024;
                while pos < data.len() {
                    let end = cmp::min(data.len(), pos + MAX_CHUNK_SIZE);

                    let end_stream =
                        if end == data.len() && part.last {
                            EndStream::Yes
                        } else {
                            EndStream::No
                        };

                    let mut frame = DataFrame::with_data(stream_id, &data[pos..end]);
                    if end_stream == EndStream::Yes {
                        frame.set_flag(DataFlag::EndStream);
                    }

                    debug!("sending frame {:?}", frame);

                    target.send_frame(frame).unwrap();

                    pos = end;
                }
            }
            HttpStreamPartContent::Headers(headers) => {
                let headers_fragment = self
                    .conn.encoder.encode(headers.iter().map(|h| (h.name(), h.value())));

                // For now, sending header fragments larger than 16kB is not supported
                // (i.e. the encoded representation cannot be split into CONTINUATION
                // frames).
                let mut frame = HeadersFrame::new(headers_fragment, stream_id);
                frame.set_flag(HeadersFlag::EndHeaders);

                if part.last {
                    frame.set_flag(HeadersFlag::EndStream);
                }

                debug!("sending frame {:?}", frame);

                target.send_frame(frame).unwrap();
            }
        }
    }

    pub fn pop_outg_all_for_stream_bytes(&mut self, stream_id: StreamId) -> Vec<u8> {
        let mut send = VecSendFrame(Vec::new());
        for part in self.pop_outg_all_for_stream(stream_id) {
            self.write_part(&mut send, stream_id, part);
        }
        send.0
    }

    pub fn pop_outg_all_for_conn_bytes(&mut self) -> Vec<u8> {
        let mut send = VecSendFrame(Vec::new());
        for (stream_id, part) in self.pop_outg_all_for_conn() {
            self.write_part(&mut send, stream_id, part);
        }
        send.0
    }

    pub fn dump_state(&self) -> ConnectionStateSnapshot {
        ConnectionStateSnapshot {
            streams: self.streams.iter().map(|(&k, s)| (k, s.common().state)).collect(),
        }
    }
}


pub trait LoopInner: 'static {
    type LoopHttpStream : HttpStream;

    fn common(&mut self) -> &mut LoopInnerCommon<Self::LoopHttpStream>;

    /// Send a frame back to the network
    /// Must not be data frame
    fn send_frame<R : FrameIR>(&mut self, frame: R) {
        let mut send_buf = VecSendFrame(Vec::new());
        send_buf.send_frame(frame).unwrap();
        self.send_common(CommonToWriteMessage::Write(send_buf.0))
    }

    fn send_common(&mut self, message: CommonToWriteMessage);

    fn out_window_increased(&mut self, stream_id: Option<StreamId>) {
        self.send_common(CommonToWriteMessage::TryFlushStream(stream_id))
    }

    /// Sends an SETTINGS Frame with ack set to acknowledge seeing a SETTINGS frame from the peer.
    fn ack_settings(&mut self) {
        self.send_frame(SettingsFrame::new_ack());
    }

    fn process_headers_frame(&mut self, frame: HeadersFrame);

    fn process_settings_global(&mut self, _frame: SettingsFrame) {
        // TODO: apply settings

        self.ack_settings();
    }

    fn process_stream_window_update_frame(&mut self, frame: WindowUpdateFrame) {
        {
            match self.common().get_stream_mut(frame.get_stream_id()) {
                Some(stream) => {
                    stream.common_mut().out_window_size.try_increase(frame.increment())
                        .expect("failed to increment stream window");
                }
                None => {
                    // 6.9
                    // WINDOW_UPDATE can be sent by a peer that has sent a frame bearing the
                    // END_STREAM flag.  This means that a receiver could receive a
                    // WINDOW_UPDATE frame on a "half-closed (remote)" or "closed" stream.
                    // A receiver MUST NOT treat this as an error (see Section 5.1).
                    debug!("WINDOW_UPDATE of unknown stream: {}", frame.get_stream_id());
                }
            }
        }
        self.out_window_increased(Some(frame.get_stream_id()));
    }

    fn process_conn_window_update(&mut self, frame: WindowUpdateFrame) {
        self.common().conn.out_window_size.try_increase(frame.increment())
            .expect("failed to increment conn window");
        self.out_window_increased(None);
    }

    fn process_rst_stream_frame(&mut self, _frame: RstStreamFrame) {
    }

    fn process_data_frame(&mut self, frame: DataFrame) {
        let stream_id = frame.get_stream_id();

        self.common().conn.decrease_in_window(frame.payload_len())
            .expect("failed to decrease conn win");

        let increment_conn =
            if self.common().conn.in_window_size() < INITIAL_CONNECTION_WINDOW_SIZE / 2 {
                let increment = INITIAL_CONNECTION_WINDOW_SIZE as u32;
                self.common().conn.in_window_size.try_increase(increment).expect("failed to increase");

                Some(increment)
            } else {
                None
            };

        let increment_stream = {
            let stream = self.common().get_stream_mut(frame.get_stream_id())
                .expect(&format!("stream not found: {}", frame.get_stream_id()));

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

    fn process_goaway(&mut self, _frame: GoawayFrame) {
        // TODO: After all streams end, close the connection.
    }

    fn process_conn_frame(&mut self, frame: HttpFrameConn) {
        match frame {
            HttpFrameConn::Settings(f) => self.process_settings_global(f),
            HttpFrameConn::Ping(f) => self.process_ping(f),
            HttpFrameConn::Goaway(f) => self.process_goaway(f),
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

        // RST_STREAM and WINDOW_UPDATE can be received when we have no stream
        {
            if let Some(stream) = self.common().get_stream_mut(stream_id) {
                stream.common_mut().close_remote();
                stream.closed_remote();
            } else {
                return;
            }
        }

        self.common().remove_stream_if_closed(stream_id);
    }
}


pub struct ReadLoopData<I, N>
    where
        I : Io + 'static,
        N : LoopInner,
{
    pub read: ReadHalf<I>,
    pub inner: TaskRcMut<N>,
}

pub struct WriteLoopData<I, N>
    where
        I : Io + 'static,
        N : LoopInner,
{
    pub write: WriteHalf<I>,
    pub inner: TaskRcMut<N>,
}

pub struct CommandLoopData<N>
    where
        N : LoopInner,
{
    pub inner: TaskRcMut<N>,
}


impl<I, N> ReadLoopData<I, N>
    where
        I : Io + Send + 'static,
        N : LoopInner,
{
    /// Recv a frame from the network
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame)> {
        let ReadLoopData { read, inner } = self;
        Box::new(recv_raw_frame(read)
            .map(|(read, frame)| (ReadLoopData { read: read, inner: inner }, frame))
            .map_err(HttpError::from))
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        Box::new(self.recv_raw_frame()
            .and_then(move |(lp, frame)| lp.process_raw_frame(frame)))
    }

    pub fn run(self) -> HttpFuture<()> {
        let future = future::loop_fn(self, |lp| {
            lp.read_process_frame().map(future::Loop::Continue::<(), _>)
        });

        Box::new(future)
    }

    fn process_raw_frame(self, frame: RawFrame) -> HttpFuture<Self> {
        self.inner.with(move |inner| {
            inner.process_raw_frame(frame);
        });
        Box::new(futures::finished(self))
    }

}

impl<I, N> WriteLoopData<I, N>
    where
        I : Io + Send + 'static,
        N : LoopInner,
{
    pub fn write_all(self, buf: Vec<u8>) -> HttpFuture<Self> {
        let WriteLoopData { write, inner } = self;

        Box::new(tokio_io::write_all(write, buf)
            .map(move |(write, _)| WriteLoopData { write: write, inner: inner })
            .map_err(HttpError::from))
    }

    fn with_inner<G, R>(&self, f: G) -> R
        where G: FnOnce(&mut N) -> R
    {
        self.inner.with(f)
    }

    pub fn send_outg_stream(self, stream_id: StreamId) -> HttpFuture<Self> {
        let bytes = self.with_inner(|inner| {
            inner.common().pop_outg_all_for_stream_bytes(stream_id)
        });

        self.write_all(bytes)
    }

    fn send_outg_conn(self) -> HttpFuture<Self> {
        let bytes = self.with_inner(|inner| {
            inner.common().pop_outg_all_for_conn_bytes()
        });

        self.write_all(bytes)
    }

    pub fn process_common(self, common: CommonToWriteMessage) -> HttpFuture<Self> {
        match common {
            CommonToWriteMessage::TryFlushStream(None) => self.send_outg_conn(),
            CommonToWriteMessage::TryFlushStream(Some(stream_id)) => self.send_outg_stream(stream_id),
            CommonToWriteMessage::Write(buf) => self.write_all(buf),
        }
    }
}

impl<N> CommandLoopData<N>
    where
        N : LoopInner,
{
}
