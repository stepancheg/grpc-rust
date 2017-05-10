//! The module contains the implementation of an HTTP/2 connection.
//!
//! This provides an API to read and write raw HTTP/2 frames, as well as a way to hook into
//! higher-level events arising on an HTTP/2 connection, such as the receipt of headers on a
//! particular stream or a new data chunk.
//!
//! The `SendFrame` and `ReceiveFrame` traits are the API to sending and receiving frames off of an
//! HTTP/2 connection. The module includes default implementations of those traits for `io::Write`
//! and `solicit::http::transport::TransportStream` types.
//!
//! The `HttpConnection` struct builds on top of these traits and provides an API for sending
//! messages of a higher level to the peer (such as writing data or headers, while automatically
//! handling the framing and header encoding), as well as for handling incoming events of that
//! type. The `Session` trait is the bridge between the connection layer (i.e. the
//! `HttpConnection`) and the higher layers that handle these events and pass them on to the
//! application.

use std::borrow::Cow;
use std::borrow::Borrow;

use solicit::{StreamId, HttpError, HttpResult, HttpScheme, WindowSize,
           ErrorCode, INITIAL_CONNECTION_WINDOW_SIZE};
use solicit::header::Header;
use solicit::frame::{Frame, FrameIR, RawFrame, DataFrame, DataFlag, HeadersFrame, HeadersFlag,
                  SettingsFrame, RstStreamFrame, PingFrame, GoawayFrame, WindowUpdateFrame};
use hpack;

#[derive(Debug)]
pub enum HttpFrameType {
    Data,
    Headers,
    RstStream,
    Settings,
    Ping,
    Goaway,
    WindowUpdate,
    Unknown(u8),
}

/// An enum representing all frame variants that can be returned by an `HttpConnection` can handle.
///
/// The variants wrap the appropriate `Frame` implementation, except for the `UnknownFrame`
/// variant, which provides an owned representation of the underlying `RawFrame`
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum HttpFrame {
    DataFrame(DataFrame),
    HeadersFrame(HeadersFrame),
    RstStreamFrame(RstStreamFrame),
    SettingsFrame(SettingsFrame),
    PingFrame(PingFrame),
    GoawayFrame(GoawayFrame),
    WindowUpdateFrame(WindowUpdateFrame),
    UnknownFrame(RawFrame),
}

impl HttpFrame{
    pub fn from_raw(raw_frame: &RawFrame) -> HttpResult<HttpFrame> {
        let frame = match raw_frame.header().1 {
            0x0 => HttpFrame::DataFrame(HttpFrame::parse_frame(&raw_frame)?),
            0x1 => HttpFrame::HeadersFrame(HttpFrame::parse_frame(&raw_frame)?),
            0x3 => HttpFrame::RstStreamFrame(HttpFrame::parse_frame(&raw_frame)?),
            0x4 => HttpFrame::SettingsFrame(HttpFrame::parse_frame(&raw_frame)?),
            0x6 => HttpFrame::PingFrame(HttpFrame::parse_frame(&raw_frame)?),
            0x7 => HttpFrame::GoawayFrame(HttpFrame::parse_frame(&raw_frame)?),
            0x8 => HttpFrame::WindowUpdateFrame(HttpFrame::parse_frame(&raw_frame)?),
            _ => HttpFrame::UnknownFrame(raw_frame.as_ref().into()),
        };

        Ok(frame)
    }

    /// A helper method that parses the given `RawFrame` into the given `Frame`
    /// implementation.
    ///
    /// # Returns
    ///
    /// Failing to decode the given `Frame` from the `raw_frame`, an
    /// `HttpError::InvalidFrame` error is returned.
    #[inline]
    fn parse_frame<F: Frame>(raw_frame: &RawFrame) -> HttpResult<F> {
        // TODO: The reason behind being unable to decode the frame should be
        //       extracted to allow an appropriate connection-level action to be
        //       taken (e.g. responding with a PROTOCOL_ERROR).
        Frame::from_raw(&raw_frame).ok_or(HttpError::InvalidFrame("failed to parse frame".to_owned()))
    }

    /// Get stream id, zero for special frames
    pub fn get_stream_id(&self) -> StreamId {
        match self {
            &HttpFrame::DataFrame(ref f) => f.get_stream_id(),
            &HttpFrame::HeadersFrame(ref f) => f.get_stream_id(),
            &HttpFrame::RstStreamFrame(ref f) => f.get_stream_id(),
            &HttpFrame::SettingsFrame(ref f) => f.get_stream_id(),
            &HttpFrame::PingFrame(ref f) => f.get_stream_id(),
            &HttpFrame::GoawayFrame(ref f) => f.get_stream_id(),
            &HttpFrame::WindowUpdateFrame(ref f) => f.get_stream_id(),
            &HttpFrame::UnknownFrame(ref f) => f.get_stream_id(),
        }
    }

    pub fn frame_type(&self) -> HttpFrameType {
        match self {
            &HttpFrame::DataFrame(..) => HttpFrameType::Data,
            &HttpFrame::HeadersFrame(..) => HttpFrameType::Headers,
            &HttpFrame::RstStreamFrame(..) => HttpFrameType::RstStream,
            &HttpFrame::SettingsFrame(..) => HttpFrameType::Settings,
            &HttpFrame::PingFrame(..) => HttpFrameType::Ping,
            &HttpFrame::GoawayFrame(..) => HttpFrameType::Goaway,
            &HttpFrame::WindowUpdateFrame(..) => HttpFrameType::WindowUpdate,
            &HttpFrame::UnknownFrame(ref f) => HttpFrameType::Unknown(f.frame_type()),
        }
    }
}

/// The struct implements the HTTP/2 connection level logic.
///
/// This means that the struct is a bridge between the low level raw frame reads/writes (i.e. what
/// the `SendFrame` and `ReceiveFrame` traits do) and the higher session-level logic.
///
/// Therefore, it provides an API that exposes higher-level write operations, such as writing
/// headers or data, that take care of all the underlying frame construction that is required.
///
/// Similarly, it provides an API for handling events that arise from receiving frames, without
/// requiring the higher level to directly look at the frames themselves, rather only the semantic
/// content within the frames.
pub struct HttpConnection {
    /// HPACK decoder used to decode incoming headers before passing them on to the session.
    pub decoder: hpack::Decoder<'static>,
    /// The HPACK encoder used to encode headers before sending them on this connection.
    pub encoder: hpack::Encoder<'static>,
    /// Tracks the size of the outbound flow control window
    pub out_window_size: WindowSize,
    /// Tracks the size of the inbound flow control window
    pub in_window_size: WindowSize,
    /// Initial stream window size
    pub initial_out_window_size: u32,
    /// The scheme of the connection
    pub scheme: HttpScheme,
}

/// A trait that should be implemented by types that can provide the functionality
/// of sending HTTP/2 frames.
pub trait SendFrame {
    /// Queue the given frame for immediate sending to the peer. It is the responsibility of each
    /// individual `SendFrame` implementation to correctly serialize the given `FrameIR` into an
    /// appropriate buffer and make sure that the frame is subsequently eventually pushed to the
    /// peer.
    fn send_frame<F: FrameIR>(&mut self, frame: F) -> HttpResult<()>;
}

/// A trait that should be implemented by types that can provide the functionality
/// of receiving HTTP/2 frames.
pub trait ReceiveFrame {
    /// Return a new `HttpFrame` instance. Unknown frames can be wrapped in the
    /// `HttpFrame::UnknownFrame` variant (i.e. their `RawFrame` representation).
    fn recv_frame(&mut self) -> HttpResult<HttpFrame>;
}

/// The struct represents a chunk of data that should be sent to the peer on a particular stream.
pub struct DataChunk<'a> {
    /// The data that should be sent.
    pub data: Cow<'a, [u8]>,
    /// The ID of the stream on which the data should be sent.
    pub stream_id: StreamId,
    /// Whether the data chunk will also end the stream.
    pub end_stream: EndStream,
}

impl<'a> DataChunk<'a> {
    /// Creates a new `DataChunk`.
    ///
    /// **Note:** `IntoCow` is unstable and there's no implementation of `Into<Cow<'a, [u8]>>` for
    /// the fundamental types, making this a bit of a clunky API. Once such an `Into` impl is
    /// added, this can be made generic over the trait for some ergonomic improvements.
    pub fn new(data: Cow<'a, [u8]>, stream_id: StreamId, end_stream: EndStream) -> DataChunk<'a> {
        DataChunk {
            data: data,
            stream_id: stream_id,
            end_stream: end_stream,
        }
    }

    /// Creates a new `DataChunk` from a borrowed slice. This method should become obsolete if we
    /// can take an `Into<Cow<_, _>>` without using unstable features.
    pub fn new_borrowed<D: Borrow<&'a [u8]>>(data: D,
                                             stream_id: StreamId,
                                             end_stream: EndStream)
                                             -> DataChunk<'a> {
        DataChunk {
            data: Cow::Borrowed(data.borrow()),
            stream_id: stream_id,
            end_stream: end_stream,
        }
    }
}

/// An enum indicating whether the `HttpConnection` send operation should end the stream.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EndStream {
    /// The stream should be closed
    Yes,
    /// The stream should still be kept open
    No,
}

/// The struct represents an `HttpConnection` that has been bound to a `SendFrame` reference,
/// allowing it to send frames. It exposes convenience methods for various send operations that can
/// be invoked on the underlying stream. The methods prepare the appropriate frames and queue their
/// sending on the referenced `SendFrame` instance.
///
/// The only way for clients to obtain an `HttpConnectionSender` is to invoke the
/// `HttpConnection::sender` method and provide it a reference to the `SendFrame` that should be
/// used.
pub struct HttpConnectionSender<'a, S>
    where S: SendFrame + 'a
{
    sender: &'a mut S,
    conn: &'a mut HttpConnection,
}

impl<'a, S> HttpConnectionSender<'a, S>
    where S: SendFrame + 'a
{
    /// Sends the given frame to the peer.
    ///
    /// # Returns
    ///
    /// Any IO errors raised by the underlying transport layer are wrapped in a
    /// `HttpError::IoError` variant and propagated upwards.
    ///
    /// If the frame is successfully written, returns a unit Ok (`Ok(())`).
    #[inline]
    fn send_frame<F: FrameIR>(&mut self, frame: F) -> HttpResult<()> {
        self.sender.send_frame(frame)
    }

    /// Send a RST_STREAM frame for the given frame id
    pub fn send_rst_stream(&mut self, id: StreamId, code: ErrorCode) -> HttpResult<()> {
        self.send_frame(RstStreamFrame::new(id, code))
    }

    /// Sends a SETTINGS acknowledge frame to the peer.
    pub fn send_settings_ack(&mut self) -> HttpResult<()> {
        self.send_frame(SettingsFrame::new_ack())
    }

    /// Sends a PING ack
    pub fn send_ping_ack(&mut self, bytes: u64) -> HttpResult<()> {
        self.send_frame(PingFrame::new_ack(bytes))
    }

    /// Sends a PING request
    pub fn send_ping(&mut self, bytes: u64) -> HttpResult<()> {
        self.send_frame(PingFrame::with_data(bytes))
    }

    /// A helper function that inserts the frames required to send the given headers onto the
    /// `SendFrame` stream.
    ///
    /// The `HttpConnection` performs the HPACK encoding of the header block using an internal
    /// encoder.
    ///
    /// # Parameters
    ///
    /// - `headers` - a headers list that should be sent.
    /// - `stream_id` - the ID of the stream on which the headers will be sent. The connection
    ///   performs no checks as to whether the stream is a valid identifier.
    /// - `end_stream` - whether the stream should be closed from the peer's side immediately
    ///   after sending the headers
    pub fn send_headers<H: Into<Vec<Header>>>(
        &mut self,
        headers: H,
        stream_id: StreamId,
        end_stream: EndStream)
            -> HttpResult<()>
    {
        let headers_fragment = self.conn
                                   .encoder
                                   .encode(headers.into().iter().map(|h| (h.name(), h.value())));
        // For now, sending header fragments larger than 16kB is not supported
        // (i.e. the encoded representation cannot be split into CONTINUATION
        // frames).
        let mut frame = HeadersFrame::new(headers_fragment, stream_id);
        frame.set_flag(HeadersFlag::EndHeaders);

        if end_stream == EndStream::Yes {
            frame.set_flag(HeadersFlag::EndStream);
        }

        self.send_frame(frame)
    }

    /// A helper function that inserts a frame representing the given data into the `SendFrame`
    /// stream. In doing so, the connection's outbound flow control window is adjusted
    /// appropriately.
    pub fn send_data(&mut self, chunk: DataChunk) -> HttpResult<()> {
        // Prepare the frame...
        let DataChunk { data, stream_id, end_stream } = chunk;
        let mut frame = DataFrame::with_data(stream_id, data.as_ref());
        if end_stream == EndStream::Yes {
            frame.set_flag(DataFlag::EndStream);
        }
        // Adjust the flow control window...
        self.conn.decrease_out_window(frame.payload_len())?;
        trace!("New OUT WINDOW size = {}", self.conn.out_window_size());
        // ...and now send it out.
        self.send_frame(frame)
    }
}

impl HttpConnection {
    /// Creates a new `HttpConnection` that will use the given sender
    /// for writing frames.
    pub fn new(scheme: HttpScheme) -> HttpConnection {
        HttpConnection {
            scheme: scheme,
            decoder: hpack::Decoder::new(),
            encoder: hpack::Encoder::new(),
            initial_out_window_size: INITIAL_CONNECTION_WINDOW_SIZE as u32,
            in_window_size: WindowSize::new(INITIAL_CONNECTION_WINDOW_SIZE),
            out_window_size: WindowSize::new(INITIAL_CONNECTION_WINDOW_SIZE),
        }
    }

    /// Creates a new `HttpConnectionSender` instance that will use the given `SendFrame` instance
    /// to send the frames that it prepares. This is a convenience struct so that clients do not
    /// have to pass the same `sender` reference to multiple send methods.
    /// ```
    pub fn sender<'a, S: SendFrame>(&'a mut self, sender: &'a mut S) -> HttpConnectionSender<S> {
        HttpConnectionSender {
            sender: sender,
            conn: self,
        }
    }

    /// Returns the current size of the inbound flow control window (i.e. the number of octets that
    /// the connection will accept and the peer will send at most, unless the window is updated).
    pub fn in_window_size(&self) -> i32 {
        self.in_window_size.size()
    }
    /// Returns the current size of the outbound flow control window (i.e. the number of octets
    /// that can be sent on the connection to the peer without violating flow control).
    pub fn out_window_size(&self) -> i32 {
        self.out_window_size.size()
    }

    /// Internal helper method that decreases the outbound flow control window size.
    pub fn decrease_out_window(&mut self, size: u32) -> HttpResult<()> {
        // The size by which we decrease the window must be at most 2^31 - 1. We should be able to
        // reach here only after sending a DATA frame, whose payload also cannot be larger than
        // that, but we assert it just in case.
        debug_assert!(size < 0x80000000);
        self.out_window_size
            .try_decrease(size as i32)
            .map_err(|_| HttpError::WindowSizeOverflow)
    }

    /// Internal helper method that decreases the inbound flow control window size.
    pub fn decrease_in_window(&mut self, size: u32) -> HttpResult<()> {
        // The size by which we decrease the window must be at most 2^31 - 1. We should be able to
        // reach here only after receiving a DATA frame, which would have been validated when
        // parsed from the raw frame to have the correct payload size, but we assert it just in
        // case.
        debug_assert!(size < 0x80000000);
        self.in_window_size
            .try_decrease(size as i32)
            .map_err(|_| HttpError::WindowSizeOverflow)
    }
}
