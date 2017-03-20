//! Implements the `WINDOW_UPDATE` HTTP/2 frame.

use std::io;

use solicit::StreamId;
use solicit::frame::{Frame, FrameIR, FrameBuilder, FrameHeader, RawFrame};
use solicit::frame::flags::*;

/// The minimum size for the `WINDOW_UPDATE` frame payload.
pub const WINDOW_UPDATE_FRAME_LEN: u32 = 4;
/// The frame type of the `WINDOW_UPDATE` frame.
pub const WINDOW_UPDATE_FRAME_TYPE: u8 = 0x8;

/// The struct represents the `WINDOW_UPDATE` HTTP/2 frame.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowUpdateFrame {
    stream_id: StreamId,
    increment: u32,
    flags: Flags<NoFlag>,
}

impl WindowUpdateFrame {
    /// Creates a new `WindowUpdateFrame` that will increment the connection-level window by the
    /// given increment.
    pub fn for_connection(increment: u32) -> WindowUpdateFrame {
        WindowUpdateFrame {
            stream_id: 0,
            increment: increment,
            flags: Flags::default(),
        }
    }

    /// Creates a new `WindowUpdateFrame` that will increment the given stream's window by the
    /// given increment.
    pub fn for_stream(stream_id: StreamId, increment: u32) -> WindowUpdateFrame {
        WindowUpdateFrame {
            stream_id: stream_id,
            increment: increment,
            flags: Flags::default(),
        }
    }

    /// Returns the window increment indicated by the frame.
    pub fn increment(&self) -> u32 {
        self.increment
    }
}

impl Frame for WindowUpdateFrame {
    type FlagType = NoFlag;

    fn from_raw(raw_frame: &RawFrame) -> Option<Self> {
        let (payload_len, frame_type, flags, stream_id) = raw_frame.header();
        if payload_len != WINDOW_UPDATE_FRAME_LEN {
            return None;
        }
        if frame_type != WINDOW_UPDATE_FRAME_TYPE {
            return None;
        }

        let num = unpack_octets_4!(raw_frame.payload(), 0, u32);
        // Clear the reserved most-significant bit
        let increment = num & !0x80000000;

        Some(WindowUpdateFrame {
            stream_id: stream_id,
            increment: increment,
            flags: Flags::new(flags),
        })
    }

    fn is_set(&self, _: NoFlag) -> bool {
        false
    }
    fn get_stream_id(&self) -> StreamId {
        self.stream_id
    }
    fn get_header(&self) -> FrameHeader {
        (WINDOW_UPDATE_FRAME_LEN,
         WINDOW_UPDATE_FRAME_TYPE,
         self.flags.0,
         self.get_stream_id())
    }
}

impl FrameIR for WindowUpdateFrame {
    fn serialize_into<B: FrameBuilder>(self, builder: &mut B) -> io::Result<()> {
        builder.write_header(self.get_header())?;
        builder.write_u32(self.increment)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::WindowUpdateFrame;

    use solicit::tests::common::{serialize_frame, raw_frame_from_parts};
    use solicit::frame::Frame;

    #[test]
    fn test_parse_valid_connection_level() {
        let raw = raw_frame_from_parts((4, 0x8, 0, 0), vec![0, 0, 0, 1]);
        let frame = WindowUpdateFrame::from_raw(&raw).expect("expected valid WINDOW_UPDATE");
        assert_eq!(frame.increment(), 1);
        assert_eq!(frame.get_stream_id(), 0);
    }

    #[test]
    fn test_parse_valid_max_increment() {
        let raw = raw_frame_from_parts((4, 0x8, 0, 0), vec![0xff, 0xff, 0xff, 0xff]);
        let frame = WindowUpdateFrame::from_raw(&raw).expect("valid WINDOW_UPDATE");
        // Automatically ignores the reserved bit...
        assert_eq!(frame.increment(), 0x7FFFFFFF);
    }

    #[test]
    fn test_parse_valid_stream_level() {
        let raw = raw_frame_from_parts((4, 0x8, 0, 1), vec![0, 0, 0, 1]);
        let frame = WindowUpdateFrame::from_raw(&raw).expect("expected valid WINDOW_UPDATE");
        assert_eq!(frame.increment(), 1);
        assert_eq!(frame.get_stream_id(), 1);
    }

    /// The frame leaves it up to the higher levels to indicate the appropriate error if the
    /// increment is invalid.
    #[test]
    fn test_parse_increment_zero() {
        let raw = raw_frame_from_parts((4, 0x8, 0, 1), vec![0, 0, 0, 0]);
        let frame = WindowUpdateFrame::from_raw(&raw).expect("expected valid WINDOW_UPDATE");
        assert_eq!(frame.increment(), 0);
        assert_eq!(frame.get_stream_id(), 1);
    }

    #[test]
    fn test_serialize_connection_level() {
        let frame = WindowUpdateFrame::for_connection(10);
        let expected: Vec<u8> = raw_frame_from_parts((4, 0x8, 0, 0), vec![0, 0, 0, 10]).as_ref().to_owned();
        let serialized = serialize_frame(&frame);

        assert_eq!(expected, serialized);
    }

    #[test]
    fn test_serialize_stream_level() {
        let frame = WindowUpdateFrame::for_stream(1, 10);
        let expected = raw_frame_from_parts((4, 0x8, 0, 1), vec![0, 0, 0, 10]).as_ref().to_owned();
        let serialized = serialize_frame(&frame);

        assert_eq!(expected, serialized);
    }
}
