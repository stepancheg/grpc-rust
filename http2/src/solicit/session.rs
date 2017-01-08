//! Defines the interface for the session-level management of HTTP/2
//! communication. This is effectively an API that allows hooking into an
//! HTTP/2 connection in order to handle events arising on the connection.
//!
//! The module also provides a default implementation for some of the traits.

/// The enum represents all the states that an HTTP/2 stream can be found in.
///
/// Corresponds to [section 5.1.](http://http2.github.io/http2-spec/#rfc.section.5.1) of the spec.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StreamState {
    Idle,
    ReservedLocal,
    ReservedRemote,
    Open,
    HalfClosedRemote,
    HalfClosedLocal,
    Closed,
}

impl StreamState {
    /// Returns whether the stream is closed.
    ///
    /// A stream is considered to be closed iff its state is set to `Closed`.
    pub fn is_closed(&self) -> bool {
        *self == StreamState::Closed
    }

    /// Returns whether the stream is closed locally.
    pub fn is_closed_local(&self) -> bool {
        match *self {
            StreamState::HalfClosedLocal | StreamState::Closed => true,
            _ => false,
        }
    }

    /// Returns whether the remote peer has closed the stream. This includes a fully closed stream.
    pub fn is_closed_remote(&self) -> bool {
        match *self {
            StreamState::HalfClosedRemote | StreamState::Closed => true,
            _ => false,
        }
    }
}
