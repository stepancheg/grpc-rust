//! The module contains some common utilities for `solicit::http` tests.

use std::io;
use std::io::Write;

use solicit::frame::{RawFrame, FrameIR, FrameHeader, pack_header};


/// Creates a new `RawFrame` from two separate parts: the header and the payload.
/// Useful for tests that need to create frames, since they can easily specify the header and the
/// payload separately and use this function to stitch them together into a `RawFrame`.
pub fn raw_frame_from_parts(header: FrameHeader, payload: Vec<u8>) -> RawFrame {
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
