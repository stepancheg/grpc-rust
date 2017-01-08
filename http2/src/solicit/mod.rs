//! The module implements the framing layer of HTTP/2 and exposes an API for using it.
use std::io;
use std::fmt;
use std::borrow::Cow;
use std::borrow::Borrow;
use std::convert::From;
use std::error::Error;

use hpack::decoder::DecoderError;

pub mod frame;
pub mod connection;
pub mod session;

/// The initial size of the connections' flow control window.
pub const INITIAL_CONNECTION_WINDOW_SIZE: i32 = 65_535;

/// An alias for the type that represents the ID of an HTTP/2 stream
pub type StreamId = u32;
/// An alias for the type that represents an HTTP/2 header where both the name and the value is
/// owned.
pub type OwnedHeader = (Vec<u8>, Vec<u8>);

/// A convenience struct representing a part of a header (either the name or the value) that can be
/// either an owned or a borrowed byte sequence.
pub struct HeaderPart<'a>(Cow<'a, [u8]>);

impl<'a> fmt::Debug for HeaderPart<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "b\""));
        let u8a: &[u8] = self.0.borrow();
        for &c in u8a {
            // ASCII printable
            if c >= 0x20 && c < 0x7f {
                try!(write!(fmt, "{}", c as char));
            } else {
                try!(write!(fmt, "\\x{:02x}", c));
            }
        }
        try!(write!(fmt, "\""));
        Ok(())
    }
}

impl<'a> From<Vec<u8>> for HeaderPart<'a> {
    fn from(vec: Vec<u8>) -> HeaderPart<'a> {
        HeaderPart(Cow::Owned(vec))
    }
}

impl<'a> From<&'a [u8]> for HeaderPart<'a> {
    fn from(buf: &'a [u8]) -> HeaderPart<'a> {
        HeaderPart(Cow::Borrowed(buf))
    }
}

impl<'a> From<Cow<'a, [u8]>> for HeaderPart<'a> {
    fn from(cow: Cow<'a, [u8]>) -> HeaderPart<'a> {
        HeaderPart(cow)
    }
}

macro_rules! from_static_size_array {
    ($N:expr) => (
        impl<'a> From<&'a [u8; $N]> for HeaderPart<'a> {
            fn from(buf: &'a [u8; $N]) -> HeaderPart<'a> {
                buf[..].into()
            }
        }
    );
}

macro_rules! impl_from_static_size_array {
    ($($N:expr,)+) => {
        $(
            from_static_size_array!($N);
        )+
    }
}

impl_from_static_size_array!(
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
);

impl<'a> From<String> for HeaderPart<'a> {
    fn from(s: String) -> HeaderPart<'a> {
        From::from(s.into_bytes())
    }
}

impl<'a> From<&'a str> for HeaderPart<'a> {
    fn from(s: &'a str) -> HeaderPart<'a> {
        From::from(s.as_bytes())
    }
}

impl<'a> From<Cow<'a, str>> for HeaderPart<'a> {
    fn from(cow: Cow<'a, str>) -> HeaderPart<'a> {
        match cow {
            Cow::Borrowed(s) => From::from(s),
            Cow::Owned(s) => From::from(s),
        }
    }
}

impl<'n, 'v> PartialEq<Header<'n, 'v>> for OwnedHeader {
    fn eq(&self, other: &Header<'n, 'v>) -> bool {
        &self.0[..] == other.name() && &self.1[..] == other.value()
    }
}

impl<'n, 'v> PartialEq<OwnedHeader> for Header<'n, 'v> {
    fn eq(&self, other: &OwnedHeader) -> bool {
        &other.0[..] == self.name() && &other.1[..] == self.value()
    }
}

/// Represents an HTTP/2 header. Allows both the name and the value to be either an owned or a
/// borrowed byte sequence.
///
/// # Examples
///
/// A new `Header` can be created by providing an owned or borrowed name, as well as value:
///
/// ```rust
/// use httpbis::solicit::Header;
/// // Name and value both borrowed (static) slices.
/// let h1 = Header::new(b":method", b"GET");
/// assert_eq!(h1.name(), &b":method"[..]);
/// assert_eq!(h1.value(), &b"GET"[..]);
/// // A borrowed slice with a scope-bound lifetime as the value; static name
/// {
///     let value = vec![1];
///     let header = Header::new(&b"x-test-head"[..], &value[..]);
///     assert_eq!(header.name(), &b"x-test-head"[..]);
///     assert_eq!(header.value(), &[1][..]);
/// }
/// // An owned value, static name
/// {
///     let value = vec![1];
///     let header = Header::new(&b"x-test-head"[..], value);
///     assert_eq!(header.name(), &b"x-test-head"[..]);
///     assert_eq!(header.value(), &[1][..]);
/// }
/// // An owned name, as well as value
/// {
///     let value = vec![1];
///     let name = b"x-test-head".to_vec();
///     let header = Header::new(name, value);
///     assert_eq!(header.name(), &b"x-test-head"[..]);
///     assert_eq!(header.value(), &[1][..]);
/// }
/// ```
#[derive(Clone, PartialEq)]
pub struct Header<'n, 'v> {
    name: Cow<'n, [u8]>,
    value: Cow<'v, [u8]>,
}

/// A type alias for a `Header` where both the name, as well as the value must have a `'static`
/// lifetime if it is borrowed. Owned parts are allowed.
pub type StaticHeader = Header<'static, 'static>;

impl<'n, 'v> Header<'n, 'v> {
    /// Creates a new `Header` with the given name and value.
    ///
    /// The name and value need to be convertible into a `HeaderPart`.
    pub fn new<N: Into<HeaderPart<'n>>, V: Into<HeaderPart<'v>>>(name: N,
                                                                 value: V)
                                                                 -> Header<'n, 'v> {
        Header {
            name: name.into().0,
            value: value.into().0,
        }
    }

    /// Return a borrowed representation of the `Header` name.
    pub fn name(&self) -> &[u8] {
        &self.name
    }
    /// Return a borrowed representation of the `Header` value.
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

impl<'n, 'v> fmt::Debug for Header<'n, 'v> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "Header {{ name: {:?}, value: {:?} }}",
            HeaderPart::from(self.name()), HeaderPart::from(self.value()))
    }
}

impl<'n, 'v> Into<OwnedHeader> for Header<'n, 'v> {
    fn into(self) -> OwnedHeader {
        (self.name.into_owned(), self.value.into_owned())
    }
}

impl<'n, 'v> Into<Header<'n, 'v>> for OwnedHeader {
    fn into(self) -> Header<'n, 'v> {
        Header::new(self.0, self.1)
    }
}

/// A set of protocol names that the library should use to indicate that HTTP/2
/// is supported during protocol negotiation (NPN or ALPN).
/// We include some of the drafts' protocol names, since there is basically no
/// difference for all intents and purposes (and some servers out there still
/// only officially advertise draft support).
/// TODO: Eventually only use "h2".
pub const ALPN_PROTOCOLS: &'static [&'static [u8]] = &[b"h2", b"h2-16", b"h2-15", b"h2-14"];

/// The enum represents an error code that are used in `RST_STREAM` and `GOAWAY` frames.
/// These are defined in [Section 7](http://http2.github.io/http2-spec/#ErrorCodes) of the HTTP/2
/// spec.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    /// The associated condition is not a result of an error. For example, a GOAWAY might include
    /// this code to indicate graceful shutdown of a connection.
    NoError = 0x0,
    /// The endpoint detected an unspecific protocol error. This error is for use when a more
    /// specific error code is not available.
    ProtocolError = 0x1,
    /// The endpoint encountered an unexpected internal error.
    InternalError = 0x2,
    /// The endpoint detected that its peer violated the flow-control protocol.
    FlowControlError = 0x3,
    /// The endpoint sent a SETTINGS frame but did not receive a response in a timely manner. See
    /// Section 6.5.3 ("Settings Synchronization").
    SettingsTimeout = 0x4,
    /// The endpoint received a frame after a stream was half-closed.
    StreamClosed = 0x5,
    /// The endpoint received a frame with an invalid size.
    FrameSizeError = 0x6,
    /// The endpoint refused the stream prior to performing any application processing (see Section
    /// 8.1.4 for details).
    RefusedStream = 0x7,
    /// Used by the endpoint to indicate that the stream is no longer needed.
    Cancel = 0x8,
    /// The endpoint is unable to maintain the header compression context for the connection.
    CompressionError = 0x9,
    /// The connection established in response to a CONNECT request (Section 8.3) was reset or
    /// abnormally closed.
    ConnectError = 0xa,
    /// The endpoint detected that its peer is exhibiting a behavior that might be generating
    /// excessive load.
    EnhanceYourCalm = 0xb,
    /// The underlying transport has properties that do not meet minimum security requirements (see
    /// Section 9.2).
    InadequateSecurity = 0xc,
    /// The endpoint requires that HTTP/1.1 be used instead of HTTP/2.
    Http11Required = 0xd,
}

impl From<u32> for ErrorCode {
    /// Converts the given `u32` number to the appropriate `ErrorCode` variant.
    fn from(code: u32) -> ErrorCode {
        match code {
            0x0 => ErrorCode::NoError,
            0x1 => ErrorCode::ProtocolError,
            0x2 => ErrorCode::InternalError,
            0x3 => ErrorCode::FlowControlError,
            0x4 => ErrorCode::SettingsTimeout,
            0x5 => ErrorCode::StreamClosed,
            0x6 => ErrorCode::FrameSizeError,
            0x7 => ErrorCode::RefusedStream,
            0x8 => ErrorCode::Cancel,
            0x9 => ErrorCode::CompressionError,
            0xa => ErrorCode::ConnectError,
            0xb => ErrorCode::EnhanceYourCalm,
            0xc => ErrorCode::InadequateSecurity,
            0xd => ErrorCode::Http11Required,
            // According to the spec, unknown error codes MAY be treated as equivalent to
            // INTERNAL_ERROR.
            _ => ErrorCode::InternalError,
        }
    }
}

impl AsRef<str> for ErrorCode {
    fn as_ref(&self) -> &str {
        match *self {
            ErrorCode::NoError => "NoError",
            ErrorCode::ProtocolError => "ProtocolError",
            ErrorCode::InternalError => "InternalError",
            ErrorCode::FlowControlError => "FlowControlError",
            ErrorCode::SettingsTimeout => "SettingsTimeout",
            ErrorCode::StreamClosed => "StreamClosed",
            ErrorCode::FrameSizeError => "FrameSizeError",
            ErrorCode::RefusedStream => "RefusedStream",
            ErrorCode::Cancel => "Cancel",
            ErrorCode::CompressionError => "CompressionError",
            ErrorCode::ConnectError => "ConnectError",
            ErrorCode::EnhanceYourCalm => "EnhanceYourCalm",
            ErrorCode::InadequateSecurity => "InadequateSecurity",
            ErrorCode::Http11Required => "Http11Required",
        }
    }
}

impl Into<u32> for ErrorCode {
    #[inline]
    fn into(self) -> u32 {
        self as u32
    }
}

/// The struct represents a connection error arising on an HTTP/2 connection.
#[derive(Debug, PartialEq, Clone)]
pub struct ConnectionError {
    error_code: ErrorCode,
    debug_data: Option<Vec<u8>>,
}

impl ConnectionError {
    /// Creates a new `ConnectionError` with no associated debug data.
    pub fn new(error_code: ErrorCode) -> ConnectionError {
        ConnectionError {
            error_code: error_code,
            debug_data: None,
        }
    }
    /// Creates a new `ConnectionError` with the given associated debug data.
    pub fn with_debug_data(error_code: ErrorCode, debug_data: Vec<u8>) -> ConnectionError {
        ConnectionError {
            error_code: error_code,
            debug_data: Some(debug_data),
        }
    }

    /// The error code of the underlying error.
    pub fn error_code(&self) -> ErrorCode {
        self.error_code
    }
    /// The debug data attached to the connection error, if any.
    pub fn debug_data(&self) -> Option<&[u8]> {
        self.debug_data.as_ref().map(|d| d.as_ref())
    }
    /// The debug data interpreted as a string, if possible.
    pub fn debug_str(&self) -> Option<&str> {
        self.debug_data().and_then(|data| ::std::str::from_utf8(data).ok())
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "ConnectionError: {}", self.description())
    }
}

impl Error for ConnectionError {
    fn description(&self) -> &str {
        // If any debug data is present (and is a valid utf8 string), we consider this the
        // description of the error... if not, then just use the error code.
        self.debug_str().unwrap_or(self.error_code.as_ref())
    }
}

/// An enum representing errors that can arise when performing operations involving an HTTP/2
/// connection.
#[derive(Debug)]
pub enum HttpError {
    /// The underlying IO layer raised an error
    IoError(io::Error),
    /// The HTTP/2 connection received an invalid HTTP/2 frame
    InvalidFrame,
    /// The peer indicated a connection error
    PeerConnectionError(ConnectionError),
    /// The HPACK decoder was unable to decode a header chunk and raised an error.
    /// Any decoder error is fatal to the HTTP/2 connection as it means that the decoder contexts
    /// will be out of sync.
    CompressionError(DecoderError),
    /// Indicates that the local peer has discovered an overflow in the size of one of the
    /// connection flow control window, which is a connection error.
    WindowSizeOverflow,
    UnknownStreamId,
    UnableToConnect,
    MalformedResponse,
    Other(Box<Error + Send + Sync>),
}

/// Implement the trait that allows us to automatically convert `io::Error`s
/// into an `HttpError` by wrapping the given `io::Error` into an `HttpError::IoError` variant.
impl From<io::Error> for HttpError {
    fn from(err: io::Error) -> HttpError {
        HttpError::IoError(err)
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "HTTP/2 Error: {}", self.description())
    }
}

impl Error for HttpError {
    fn description(&self) -> &str {
        match *self {
            HttpError::IoError(_) => "Encountered an IO error",
            HttpError::InvalidFrame => "Encountered an invalid HTTP/2 frame",
            HttpError::PeerConnectionError(ref err) => err.description(),
            HttpError::CompressionError(_) => "Encountered an error with HPACK compression",
            HttpError::WindowSizeOverflow => "The connection flow control window overflowed",
            HttpError::UnknownStreamId => "Attempted an operation with an unknown HTTP/2 stream ID",
            HttpError::UnableToConnect => "An error attempting to establish an HTTP/2 connection",
            HttpError::MalformedResponse => "The received response was malformed",
            HttpError::Other(_) => "An unknown error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            HttpError::Other(ref e) => Some(&**e),
            HttpError::IoError(ref e) => Some(e),
            HttpError::PeerConnectionError(ref e) => Some(e),
            _ => None,
        }
    }
}

/// Implementation of the `PartialEq` trait as a convenience for tests.
#[cfg(test)]
impl PartialEq for HttpError {
    fn eq(&self, other: &HttpError) -> bool {
        match (self, other) {
            (&HttpError::IoError(ref e1), &HttpError::IoError(ref e2)) => {
                e1.kind() == e2.kind() && e1.description() == e2.description()
            }
            (&HttpError::InvalidFrame, &HttpError::InvalidFrame) => true,
            (&HttpError::CompressionError(ref e1),
             &HttpError::CompressionError(ref e2)) => e1 == e2,
            (&HttpError::UnknownStreamId, &HttpError::UnknownStreamId) => true,
            (&HttpError::UnableToConnect, &HttpError::UnableToConnect) => true,
            (&HttpError::MalformedResponse, &HttpError::MalformedResponse) => true,
            (&HttpError::Other(ref e1), &HttpError::Other(ref e2)) => {
                e1.description() == e2.description()
            }
            _ => false,
        }
    }
}

/// A convenience `Result` type that has the `HttpError` type as the error
/// type and a generic Ok result type.
pub type HttpResult<T> = Result<T, HttpError>;

/// The struct represents the size of a flow control window.
///
/// It exposes methods that allow the manipulation of window sizes, such that they can never
/// overflow the spec-mandated upper bound.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WindowSize(i32);
impl WindowSize {
    /// Tries to increase the window size by the given delta. If the WindowSize would overflow the
    /// maximum allowed value (2^31 - 1), returns an error case. If the increase succeeds, returns
    /// `Ok`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use httpbis::solicit::WindowSize;
    ///
    /// let mut window_size = WindowSize::new(65_535);
    /// assert_eq!(window_size.size(), 65_535);
    /// // An increase within the bounds...
    /// assert!(window_size.try_increase(100).is_ok());
    /// assert_eq!(window_size.size(), 65_635);
    /// // An increase that would overflow
    /// assert!(window_size.try_increase(0x7fffffff).is_err());
    /// assert_eq!(window_size.size(), 65_635);
    /// ```
    pub fn try_increase(&mut self, delta: u32) -> Result<(), ()> {
        // Someone's provided a delta that would definitely overflow the window size.
        if delta > 0x7fffffff {
            return Err(());
        }
        // Now it is safe to cast the delta to the `i32`.
        match self.0.checked_add(delta as i32) {
            None => {
                // When the add overflows, we will have went over the maximum allowed size of the
                // window size...
                Err(())
            }
            Some(next_val) => {
                // The addition didn't overflow, so the next window size is in the range allowed by
                // the spec.
                self.0 = next_val;
                Ok(())
            }
        }
    }

    /// Tries to decrease the size of the window by the given delta.
    ///
    /// There are situations where the window size should legitimately be allowed to become
    /// negative, so the only situation where the result is an error is if the window size would
    /// underflow, as this would definitely cause the peers to lose sync.
    ///
    /// # Example
    ///
    /// ```rust
    /// use httpbis::solicit::WindowSize;
    ///
    /// let mut window_size = WindowSize::new(65_535);
    /// assert_eq!(window_size.size(), 65_535);
    /// // A decrease...
    /// assert!(window_size.try_decrease(100).is_ok());
    /// assert_eq!(window_size.size(), 65_435);
    /// // A decrease that does not underflow
    /// assert!(window_size.try_decrease(0x7fffffff).is_ok());
    /// assert_eq!(window_size.size(), -2147418212);
    /// // A decrease that *would* underflow
    /// assert!(window_size.try_decrease(0x7fffffff).is_err());
    /// assert_eq!(window_size.size(), -2147418212);
    /// ```
    pub fn try_decrease(&mut self, delta: i32) -> Result<(), ()> {
        match self.0.checked_sub(delta) {
            Some(new) => {
                self.0 = new;
                Ok(())
            }
            None => Err(()),
        }
    }

    /// Creates a new `WindowSize` with the given initial size.
    pub fn new(size: i32) -> WindowSize {
        WindowSize(size)
    }
    /// Returns the current size of the window.
    ///
    /// The size is actually allowed to become negative (for instance if the peer changes its
    /// intial window size in the settings); therefore, the return is an `i32`.
    pub fn size(&self) -> i32 {
        self.0
    }
}

/// An enum representing the two possible HTTP schemes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HttpScheme {
    /// The variant corresponding to `http://`
    Http,
    /// The variant corresponding to `https://`
    Https,
}

impl HttpScheme {
    /// Returns a byte string representing the scheme.
    #[inline]
    pub fn as_bytes(&self) -> &'static [u8] {
        match *self {
            HttpScheme::Http => b"http",
            HttpScheme::Https => b"https",
        }
    }
}

/// A struct representing the full raw response received on an HTTP/2 connection.
///
/// The full body of the response is included, regardless how large it may be.
/// The headers contain both the meta-headers, as well as the actual headers.
#[derive(Clone)]
pub struct Response<'n, 'v> {
    /// The ID of the stream to which the response is associated. HTTP/1.1 does
    /// not really have an equivalent to this.
    pub stream_id: StreamId,
    /// Exposes *all* the raw response headers, including the meta-headers.
    /// (For now the only meta header allowed in HTTP/2 responses is the
    /// `:status`.)
    pub headers: Vec<Header<'n, 'v>>,
    /// The full body of the response as an uninterpreted sequence of bytes.
    pub body: Vec<u8>,
}

/// A type alias for a `Response` where all headers' names and values must have a `'static`
/// lifetime if they are borrowed. This means that the parts can also be owned.
pub type StaticResponse = Response<'static, 'static>;

impl<'n, 'v> Response<'n, 'v> {
    /// Creates a new `Response` with all the components already provided.
    pub fn new(stream_id: StreamId, headers: Vec<OwnedHeader>, body: Vec<u8>) -> Response<'n, 'v> {
        Response {
            stream_id: stream_id,
            headers: headers.into_iter().map(|h| Header::new(h.0, h.1)).collect(),
            body: body,
        }
    }

    /// Gets the response status code from the pseudo-header. If the response
    /// does not contain the response as the first pseuo-header, an error is
    /// returned as such a response is malformed.
    pub fn status_code(&self) -> HttpResult<u16> {
        // Since pseudo-headers MUST be found before any regular header fields
        // and the *only* pseudo-header defined for responses is the `:status`
        // field, the `:status` MUST be the first header; otherwise, the
        // response is malformed.
        if self.headers.len() < 1 {
            return Err(HttpError::MalformedResponse);
        }
        if &self.headers[0].name[..] == &b":status"[..] {
            Ok(try!(Response::parse_status_code(&self.headers[0].value)))
        } else {
            Err(HttpError::MalformedResponse)
        }
    }

    /// A helper function that parses a given buffer as a status code and
    /// returns it as a `u16`, if it is valid.
    fn parse_status_code(buf: &[u8]) -> HttpResult<u16> {
        // "The status-code element is a three-digit integer code [...]"
        if buf.len() != 3 {
            return Err(HttpError::MalformedResponse);
        }

        // "There are five values for the first digit"
        if buf[0] < b'1' || buf[0] > b'5' {
            return Err(HttpError::MalformedResponse);
        }

        // The rest of them just have to be digits
        if buf[1] < b'0' || buf[1] > b'9' || buf[2] < b'0' || buf[2] > b'9' {
            return Err(HttpError::MalformedResponse);
        }

        // Finally, we can merge them into an integer
        Ok(100 * ((buf[0] - b'0') as u16) + 10 * ((buf[1] - b'0') as u16) +
              ((buf[2] - b'0') as u16))
    }
}

/// A struct representing a full HTTP/2 request, along with the full body, as a
/// sequence of bytes.
#[derive(Clone)]
pub struct Request<'n, 'v> {
    pub stream_id: u32,
    pub headers: Vec<Header<'n, 'v>>,
    pub body: Vec<u8>,
}


#[cfg(test)]
pub mod tests;
