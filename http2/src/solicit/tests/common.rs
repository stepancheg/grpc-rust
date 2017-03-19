//! The module contains some common utilities for `solicit::http` tests.

use std::io;
use std::rc::Rc;
use std::cell::{RefCell, Cell};
use std::borrow::Cow;
use std::io::{Cursor, Read, Write};

use solicit::{Header, ErrorCode};
use solicit::frame::{RawFrame, FrameIR, FrameHeader, pack_header};
use solicit::session::StreamState;
use solicit::connection::{SendFrame, ReceiveFrame, HttpFrame, HttpConnection, EndStream, DataChunk};

/// Creates a new `RawFrame` from two separate parts: the header and the payload.
/// Useful for tests that need to create frames, since they can easily specify the header and the
/// payload separately and use this function to stitch them together into a `RawFrame`.
pub fn raw_frame_from_parts<'a>(header: FrameHeader, payload: Vec<u8>) -> RawFrame<'a> {
    let mut buf = Vec::new();
    assert_eq!(9, buf.write(&pack_header(&header)[..]).unwrap());
    assert_eq!(payload.len(), buf.write(&payload).unwrap());
    buf.into()
}

/// Serializes the given frame into a newly allocated vector (without consuming the frame).
pub fn serialize_frame<F: FrameIR + Clone>(frame: &F) -> Vec<u8> {
    let mut buf = io::Cursor::new(Vec::new());
    frame.clone().serialize_into(&mut buf).ok().expect("Expected the serialization to succeed");
    buf.into_inner()
}
