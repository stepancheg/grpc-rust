//! The module contains the implementation of the `RST_STREAM` frame.
use std::io;

use solicit::{ErrorCode, StreamId};
use solicit::frame::{Frame, FrameIR, FrameBuilder, FrameHeader, RawFrame};
use solicit::frame::flags::*;

/// The total allowed size for the `RST_STREAM` frame payload.
pub const RST_STREAM_FRAME_LEN: u32 = 4;
/// The frame type of the `RST_STREAM` frame.
pub const RST_STREAM_FRAME_TYPE: u8 = 0x3;

/// The struct represents the `RST_STREAM` HTTP/2 frame.
#[derive(Clone, Debug, PartialEq)]
pub struct RstStreamFrame {
    raw_error_code: u32,
    pub stream_id: StreamId,
    flags: Flags<NoFlag>,
}

impl RstStreamFrame {
    /// Constructs a new `RstStreamFrame` with the given `ErrorCode`.
    pub fn new(stream_id: StreamId, error_code: ErrorCode) -> RstStreamFrame {
        RstStreamFrame {
            raw_error_code: error_code.into(),
            stream_id: stream_id,
            flags: Flags::default(),
        }
    }

    /// Constructs a new `RstStreamFrame` that will use the given `raw_error_code` for its payload.
    pub fn with_raw_error_code(stream_id: StreamId, raw_error_code: u32) -> RstStreamFrame {
        RstStreamFrame {
            raw_error_code: raw_error_code,
            stream_id: stream_id,
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
}

impl Frame for RstStreamFrame {
    type FlagType = NoFlag;

    fn from_raw(raw_frame: &RawFrame) -> Option<Self> {
        let (payload_len, frame_type, flags, stream_id) = raw_frame.header();
        if payload_len != RST_STREAM_FRAME_LEN {
            return None;
        }
        if frame_type != RST_STREAM_FRAME_TYPE {
            return None;
        }
        if stream_id == 0x0 {
            return None;
        }

        let error = unpack_octets_4!(raw_frame.payload(), 0, u32);

        Some(RstStreamFrame {
            raw_error_code: error,
            stream_id: stream_id,
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
        (RST_STREAM_FRAME_LEN,
         RST_STREAM_FRAME_TYPE,
         self.flags.0,
         self.stream_id)
    }
}

impl FrameIR for RstStreamFrame {
    fn serialize_into<B: FrameBuilder>(self, builder: &mut B) -> io::Result<()> {
        builder.write_header(self.get_header())?;
        builder.write_u32(self.raw_error_code)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RstStreamFrame;

    use solicit::tests::common::serialize_frame;
    use solicit::ErrorCode;
    use solicit::frame::{pack_header, FrameHeader, Frame};

    /// A helper function that creates a new Vec containing the serialized representation of the
    /// given `FrameHeader` followed by the raw provided payload.
    fn prepare_frame_bytes(header: FrameHeader, payload: Vec<u8>) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(pack_header(&header).to_vec());
        buf.extend(payload);
        buf
    }

    #[test]
    fn test_parse_valid() {
        let raw = prepare_frame_bytes((4, 0x3, 0, 1), vec![0, 0, 0, 1]);
        let rst = RstStreamFrame::from_raw(&raw.into()).expect("Valid frame expected");
        assert_eq!(rst.error_code(), ErrorCode::ProtocolError);
        assert_eq!(rst.get_stream_id(), 1);
    }

    #[test]
    fn test_parse_valid_with_unknown_flags() {
        let raw = prepare_frame_bytes((4, 0x3, 0x80, 1), vec![0, 0, 0, 1]);
        let rst = RstStreamFrame::from_raw(&raw.into()).expect("Valid frame expected");
        assert_eq!(rst.error_code(), ErrorCode::ProtocolError);
        assert_eq!(rst.get_stream_id(), 1);
        // The raw flag set is correctly reported from the header, but the parsed frame itself does
        // not know how to interpret them.
        assert_eq!(rst.get_header().2, 0x80);
    }

    #[test]
    fn test_parse_unknown_error_code() {
        let raw = prepare_frame_bytes((4, 0x3, 0x80, 1), vec![1, 0, 0, 1]);
        let rst = RstStreamFrame::from_raw(&raw.into()).expect("Valid frame expected");
        // Unknown error codes are considered equivalent to an internal error.
        assert_eq!(rst.error_code(), ErrorCode::InternalError);
        // ...but the frame still surfaces the original raw code, so that clients can handle that,
        // if they so choose.
        assert_eq!(rst.raw_error_code(), 0x01000001);
    }

    #[test]
    fn test_parse_invalid_stream_id() {
        let raw = prepare_frame_bytes((4, 0x3, 0x80, 0), vec![0, 0, 0, 1]);
        assert!(RstStreamFrame::from_raw(&raw.into()).is_none());
    }

    #[test]
    fn test_parse_invalid_payload_size() {
        let raw = prepare_frame_bytes((5, 0x3, 0x00, 2), vec![0, 0, 0, 1, 0]);
        assert!(RstStreamFrame::from_raw(&raw.into()).is_none());
    }

    #[test]
    fn test_parse_invalid_id() {
        let raw = prepare_frame_bytes((4, 0x1, 0x00, 2), vec![0, 0, 0, 1, 0]);
        assert!(RstStreamFrame::from_raw(&raw.into()).is_none());
    }

    #[test]
    fn test_serialize_protocol_error() {
        let frame = RstStreamFrame::new(1, ErrorCode::ProtocolError);
        let raw = serialize_frame(&frame);
        assert_eq!(raw, prepare_frame_bytes((4, 0x3, 0, 1), vec![0, 0, 0, 1]));
    }

    #[test]
    fn test_serialize_stream_closed() {
        let frame = RstStreamFrame::new(2, ErrorCode::StreamClosed);
        let raw = serialize_frame(&frame);
        assert_eq!(raw, prepare_frame_bytes((4, 0x3, 0, 2), vec![0, 0, 0, 0x5]));
    }

    #[test]
    fn test_serialize_raw_error_code() {
        let frame = RstStreamFrame::with_raw_error_code(3, 1024);
        let raw = serialize_frame(&frame);
        assert_eq!(raw,
                   prepare_frame_bytes((4, 0x3, 0, 3), vec![0, 0, 0x04, 0]));
    }

    #[test]
    fn test_partial_eq() {
        let frame1 = RstStreamFrame::with_raw_error_code(3, 1);
        let frame2 = RstStreamFrame::new(3, ErrorCode::ProtocolError);
        assert_eq!(frame1, frame2);
    }
}
