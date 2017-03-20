//! Implements the `GOAWAY` HTTP/2 frame.

use std::io;

use bytes::Bytes;

use solicit::{ErrorCode, StreamId};
use solicit::frame::{Frame, FrameIR, FrameBuilder, FrameHeader, RawFrame, parse_stream_id};
use solicit::frame::flags::*;

/// The minimum size for the `GOAWAY` frame payload.
/// It is 8 octets, as the last stream id and error code are required parts of the GOAWAY frame.
pub const GOAWAY_MIN_FRAME_LEN: u32 = 8;
/// The frame type of the `GOAWAY` frame.
pub const GOAWAY_FRAME_TYPE: u8 = 0x7;

/// The struct represents the `GOAWAY` HTTP/2 frame.
#[derive(Clone, Debug, PartialEq)]
pub struct GoawayFrame {
    last_stream_id: StreamId,
    raw_error_code: u32,
    debug_data: Option<Bytes>,
    flags: Flags<NoFlag>,
}

impl GoawayFrame {
    /// Create a new `GOAWAY` frame with the given error code and no debug data.
    pub fn new(last_stream_id: StreamId, error_code: ErrorCode) -> Self {
        GoawayFrame {
            last_stream_id: last_stream_id,
            raw_error_code: error_code.into(),
            debug_data: None,
            flags: Flags::default(),
        }
    }

    /// Create a new `GOAWAY` frame with the given parts.
    pub fn with_debug_data(last_stream_id: StreamId, raw_error: u32, debug_data: Bytes) -> Self {
        GoawayFrame {
            last_stream_id: last_stream_id,
            raw_error_code: raw_error,
            debug_data: Some(debug_data),
            flags: Flags::default(),
        }
    }

    /// Returns the interpreted error code of the frame. Any unknown error codes are mapped into
    /// the `InternalError` variant of the enum.
    pub fn error_code(&self) -> ErrorCode {
        self.raw_error_code.into()
    }

    /// Returns the original raw error code of the frame. If the code is unknown, it will not be
    /// changed.
    pub fn raw_error_code(&self) -> u32 {
        self.raw_error_code
    }

    /// Returns the associated last stream ID.
    pub fn last_stream_id(&self) -> StreamId {
        self.last_stream_id
    }

    /// Returns the debug data associated with the frame.
    pub fn debug_data(&self) -> Option<&Bytes> {
        self.debug_data.as_ref()
    }

    /// Returns the total length of the frame's payload, including any debug data.
    pub fn payload_len(&self) -> u32 {
        GOAWAY_MIN_FRAME_LEN + self.debug_data.as_ref().map_or(0, |d| d.len() as u32)
    }
}

impl Frame for GoawayFrame {
    type FlagType = NoFlag;

    fn from_raw(raw_frame: &RawFrame) -> Option<Self> {
        let (payload_len, frame_type, flags, stream_id) = raw_frame.header();
        if payload_len < GOAWAY_MIN_FRAME_LEN {
            return None;
        }
        if frame_type != GOAWAY_FRAME_TYPE {
            return None;
        }
        if stream_id != 0x0 {
            return None;
        }

        let last_stream_id = parse_stream_id(&raw_frame.payload());
        let error = unpack_octets_4!(raw_frame.payload(), 4, u32);
        let debug_data = if payload_len > GOAWAY_MIN_FRAME_LEN {
            Some(raw_frame.payload().slice_from(GOAWAY_MIN_FRAME_LEN as usize))
        } else {
            None
        };

        Some(GoawayFrame {
            last_stream_id: last_stream_id,
            raw_error_code: error,
            debug_data: debug_data,
            flags: Flags::new(flags),
        })
    }

    fn is_set(&self, _: NoFlag) -> bool {
        false
    }
    fn get_stream_id(&self) -> StreamId {
        0
    }
    fn get_header(&self) -> FrameHeader {
        (self.payload_len(), GOAWAY_FRAME_TYPE, self.flags.0, 0)
    }
}

impl FrameIR for GoawayFrame {
    fn serialize_into<B: FrameBuilder>(self, builder: &mut B) -> io::Result<()> {
        builder.write_header(self.get_header())?;
        builder.write_u32(self.last_stream_id)?;
        builder.write_u32(self.raw_error_code)?;
        if let Some(buf) = self.debug_data {
            builder.write_all(&buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::GoawayFrame;

    use solicit::tests::common::{serialize_frame, raw_frame_from_parts};
    use solicit::ErrorCode;
    use solicit::frame::Frame;

    use bytes::Bytes;

    #[test]
    fn test_parse_valid_no_debug_data() {
        let raw = raw_frame_from_parts((8, 0x7, 0, 0), vec![0, 0, 0, 0, 0, 0, 0, 1]);
        let frame = GoawayFrame::from_raw(&raw).expect("Expected successful parse");
        assert_eq!(frame.error_code(), ErrorCode::ProtocolError);
        assert_eq!(frame.last_stream_id(), 0);
        assert_eq!(frame.debug_data(), None);
    }

    #[test]
    fn test_parse_valid_no_debug_data_2() {
        let raw = raw_frame_from_parts((8, 0x7, 0, 0), vec![0, 0, 1, 0, 0, 0, 0, 1]);
        let frame = GoawayFrame::from_raw(&raw).expect("Expected successful parse");
        assert_eq!(frame.error_code(), ErrorCode::ProtocolError);
        assert_eq!(frame.last_stream_id(), 0x00000100);
        assert_eq!(frame.debug_data(), None);
    }

    #[test]
    fn test_parse_valid_with_debug_data() {
        let raw = raw_frame_from_parts((12, 0x7, 0, 0), vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 4]);
        let frame = GoawayFrame::from_raw(&raw).expect("Expected successful parse");
        assert_eq!(frame.error_code(), ErrorCode::ProtocolError);
        assert_eq!(frame.last_stream_id(), 0);
        assert_eq!(frame.debug_data().map(|b| &b[..]), Some(&[1, 2, 3, 4][..]));
    }

    #[test]
    fn test_parse_ignores_reserved_bit() {
        let raw = raw_frame_from_parts((8, 0x7, 0, 0), vec![0x80, 0, 0, 0, 0, 0, 0, 1]);
        let frame = GoawayFrame::from_raw(&raw).expect("Expected successful parse");
        assert_eq!(frame.error_code(), ErrorCode::ProtocolError);
        assert_eq!(frame.last_stream_id(), 0);
        assert_eq!(frame.debug_data(), None);
    }

    #[test]
    fn test_parse_invalid_id() {
        let raw = raw_frame_from_parts((12, 0x1, 0, 0), vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 4]);
        assert!(GoawayFrame::from_raw(&raw).is_none(), "expected invalid id");
    }

    #[test]
    fn test_parse_invalid_stream_id() {
        let raw = raw_frame_from_parts((8, 0x7, 0, 3), vec![0, 0, 0, 0, 0, 0, 0, 1]);
        assert!(GoawayFrame::from_raw(&raw).is_none(),
                "expected invalid stream id");
    }

    #[test]
    fn test_parse_invalid_length() {
        // Too short!
        let raw = raw_frame_from_parts((7, 0x1, 0, 0), vec![0, 0, 0, 0, 0, 0, 1]);
        assert!(GoawayFrame::from_raw(&raw).is_none(), "expected too short");
    }

    #[test]
    fn test_serialize_no_debug_data() {
        let frame = GoawayFrame::new(0, ErrorCode::ProtocolError);
        let expected: Vec<u8> = raw_frame_from_parts((8, 0x7, 0, 0), vec![0, 0, 0, 0, 0, 0, 0, 1])
                                    .as_ref().to_owned();
        let raw = serialize_frame(&frame);

        assert_eq!(expected, raw);
    }

    #[test]
    fn test_serialize_with_debug_data() {
        let frame = GoawayFrame::with_debug_data(
            0, ErrorCode::ProtocolError.into(), Bytes::from_static(b"Hi!"));
        let expected: Vec<u8> = raw_frame_from_parts((11, 0x7, 0, 0),
                                                     vec![0, 0, 0, 0, 0, 0, 0, 1, b'H', b'i',
                                                          b'!'])
                                    .as_ref().to_owned();
        let raw = serialize_frame(&frame);

        assert_eq!(expected, raw);
    }

    #[test]
    fn test_serialize_raw_error() {
        let frame = GoawayFrame::with_debug_data(1, 0x0001AA, Bytes::new());
        let expected: Vec<u8> = raw_frame_from_parts((8, 0x7, 0, 0),
                                                     vec![0, 0, 0, 1, 0, 0, 0x1, 0xAA])
                                    .as_ref().to_owned();
        let raw = serialize_frame(&frame);

        assert_eq!(expected, raw);
    }
}
