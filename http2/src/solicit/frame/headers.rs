//! The module contains the implementation of the `HEADERS` frame and associated flags.

use std::io;

use bytes::Bytes;

use solicit::StreamId;
use solicit::frame::{FrameBuilder, FrameIR, Frame, FrameHeader, RawFrame, parse_padded_payload};
use solicit::frame::flags::*;

/// An enum representing the flags that a `HeadersFrame` can have.
/// The integer representation associated to each variant is that flag's
/// bitmask.
///
/// HTTP/2 spec, section 6.2.
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Copy)]
pub enum HeadersFlag {
    EndStream = 0x1,
    EndHeaders = 0x4,
    Padded = 0x8,
    Priority = 0x20,
}

impl Flag for HeadersFlag {
    #[inline]
    fn bitmask(&self) -> u8 {
        *self as u8
    }

    fn flags() -> &'static [Self] {
        static FLAGS: &'static [HeadersFlag] = &[
            HeadersFlag::EndStream,
            HeadersFlag::EndHeaders,
            HeadersFlag::Padded,
            HeadersFlag::Priority,
        ];
        FLAGS
    }
}

/// The struct represents the dependency information that can be attached to
/// a stream and sent within a HEADERS frame (one with the Priority flag set).
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct StreamDependency {
    /// The ID of the stream that a particular stream depends on
    pub stream_id: StreamId,
    /// The weight for the stream. The value exposed (and set) here is always
    /// in the range [0, 255], instead of [1, 256] \(as defined in section 5.3.2.)
    /// so that the value fits into a `u8`.
    pub weight: u8,
    /// A flag indicating whether the stream dependency is exclusive.
    pub is_exclusive: bool,
}

impl StreamDependency {
    /// Creates a new `StreamDependency` with the given stream ID, weight, and
    /// exclusivity.
    pub fn new(stream_id: StreamId, weight: u8, is_exclusive: bool) -> StreamDependency {
        StreamDependency {
            stream_id: stream_id,
            weight: weight,
            is_exclusive: is_exclusive,
        }
    }

    /// Parses the first 5 bytes in the buffer as a `StreamDependency`.
    /// (Each 5-byte sequence is always decodable into a stream dependency
    /// structure).
    ///
    /// # Panics
    ///
    /// If the given buffer has less than 5 elements, the method will panic.
    pub fn parse(buf: &[u8]) -> StreamDependency {
        // The most significant bit of the first byte is the "E" bit indicating
        // whether the dependency is exclusive.
        let is_exclusive = buf[0] & 0x80 != 0;
        let stream_id = {
            // Parse the first 4 bytes into a u32...
            let mut id = unpack_octets_4!(buf, 0, u32);
            // ...clear the first bit since the stream id is only 31 bits.
            id &= !(1 << 31);
            id
        };

        StreamDependency {
            stream_id: stream_id,
            weight: buf[4],
            is_exclusive: is_exclusive,
        }
    }

    /// Serializes the `StreamDependency` into a 5-byte buffer representing the
    /// dependency description, as described in section 6.2. of the HTTP/2
    /// spec:
    ///
    /// ```notest
    ///  0                   1                   2                   3
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    /// +-+-------------+-----------------------------------------------+
    /// |E|                 Stream Dependency  (31)                     |
    /// +-+-------------+-----------------------------------------------+
    /// |  Weight  (8)  |
    /// +-+-------------+-----------------------------------------------+
    /// ```
    ///
    /// Where "E" is set if the dependency is exclusive.
    pub fn serialize(&self) -> [u8; 5] {
        let e_bit = if self.is_exclusive {
            1 << 7
        } else {
            0
        };
        [(((self.stream_id >> 24) & 0x000000FF) as u8) | e_bit,
         (((self.stream_id >> 16) & 0x000000FF) as u8),
         (((self.stream_id >>  8) & 0x000000FF) as u8),
         (((self.stream_id      ) & 0x000000FF) as u8),
         self.weight]
    }
}

/// A struct representing the HEADERS frames of HTTP/2, as defined in the
/// HTTP/2 spec, section 6.2.
#[derive(PartialEq, Clone, Debug)]
pub struct HeadersFrame {
    /// The set of flags for the frame, packed into a single byte.
    flags: Flags<HeadersFlag>,
    /// The ID of the stream with which this frame is associated
    pub stream_id: StreamId,
    /// The header fragment bytes stored within the frame.
    header_fragment: Bytes,
    /// The stream dependency information, if any.
    pub stream_dep: Option<StreamDependency>,
    /// The length of the padding, if any.
    pub padding_len: Option<u8>,
}

impl HeadersFrame {
    /// Creates a new `HeadersFrame` with the given header fragment and stream
    /// ID. No padding, no stream dependency, and no flags are set.
    pub fn new(fragment: Vec<u8>, stream_id: StreamId) -> HeadersFrame {
        HeadersFrame {
            header_fragment: Bytes::from(fragment),
            stream_id: stream_id,
            stream_dep: None,
            padding_len: None,
            flags: Flags::default(),
        }
    }

    /// Creates a new `HeadersFrame` with the given header fragment, stream ID
    /// and stream dependency information. No padding and no flags are set.
    pub fn with_dependency(fragment: Vec<u8>,
                           stream_id: StreamId,
                           stream_dep: StreamDependency)
                           -> HeadersFrame {
        HeadersFrame {
            header_fragment: Bytes::from(fragment),
            stream_id: stream_id,
            stream_dep: Some(stream_dep),
            padding_len: None,
            flags: HeadersFlag::Priority.to_flags(),
        }
    }

    /// Returns whether this frame ends the headers. If not, there MUST be a
    /// number of follow up CONTINUATION frames that send the rest of the
    /// header data.
    pub fn is_headers_end(&self) -> bool {
        self.is_set(HeadersFlag::EndHeaders)
    }

    /// Returns whther this frame ends the stream it is associated with.
    pub fn is_end_of_stream(&self) -> bool {
        self.is_set(HeadersFlag::EndStream)
    }

    /// Sets the padding length for the frame, as well as the corresponding
    /// Padded flag.
    pub fn set_padding(&mut self, padding_len: u8) {
        self.padding_len = Some(padding_len);
        self.set_flag(HeadersFlag::Padded);
    }

    /// Returns the length of the payload of the current frame, including any
    /// possible padding in the number of bytes.
    fn payload_len(&self) -> u32 {
        let padding = if self.is_set(HeadersFlag::Padded) {
            1 + self.padding_len.unwrap_or(0) as u32
        } else {
            0
        };
        let priority = if self.is_set(HeadersFlag::Priority) {
            5
        } else {
            0
        };

        self.header_fragment.len() as u32 + priority + padding
    }

    pub fn header_fragment(&self) -> &[u8] {
        &self.header_fragment
    }

    /// Sets the given flag for the frame.
    pub fn set_flag(&mut self, flag: HeadersFlag) {
        self.flags.set(&flag);
    }
}

impl Frame for HeadersFrame {
    /// The type that represents the flags that the particular `Frame` can take.
    /// This makes sure that only valid `Flag`s are used with each `Frame`.
    type FlagType = HeadersFlag;

    /// Creates a new `HeadersFrame` with the given `RawFrame` (i.e. header and
    /// payload), if possible.
    ///
    /// # Returns
    ///
    /// `None` if a valid `HeadersFrame` cannot be constructed from the given
    /// `RawFrame`. The stream ID *must not* be 0.
    ///
    /// Otherwise, returns a newly constructed `HeadersFrame`.
    fn from_raw(raw_frame: &RawFrame) -> Option<HeadersFrame> {
        // Unpack the header
        let (len, frame_type, flags, stream_id) = raw_frame.header();
        // Check that the frame type is correct for this frame implementation
        if frame_type != 0x1 {
            return None;
        }
        // Check that the length given in the header matches the payload
        // length; if not, something went wrong and we do not consider this a
        // valid frame.
        if (len as usize) != raw_frame.payload().len() {
            return None;
        }
        // Check that the HEADERS frame is not associated to stream 0
        if stream_id == 0 {
            return None;
        }

        // First, we get a slice containing the actual payload, depending on if
        // the frame is padded.
        let padded = (flags & HeadersFlag::Padded.bitmask()) != 0;
        let (actual, pad_len) = if padded {
            match parse_padded_payload(raw_frame.payload()) {
                Some((data, pad_len)) => (data, Some(pad_len)),
                None => return None,
            }
        } else {
            (raw_frame.payload(), None)
        };

        // From the actual payload we extract the stream dependency info, if
        // the appropriate flag is set.
        let priority = (flags & HeadersFlag::Priority.bitmask()) != 0;
        let (data, stream_dep) = if priority {
            (actual.slice_from(5), Some(StreamDependency::parse(&actual[..5])))
        } else {
            (actual, None)
        };

        Some(HeadersFrame {
            header_fragment: data,
            stream_id: stream_id,
            stream_dep: stream_dep,
            padding_len: pad_len,
            flags: Flags::new(flags),
        })
    }

    /// Tests if the given flag is set for the frame.
    fn is_set(&self, flag: HeadersFlag) -> bool {
        self.flags.is_set(&flag)
    }

    /// Returns the `StreamId` of the stream to which the frame is associated.
    ///
    /// A `SettingsFrame` always has to be associated to stream `0`.
    fn get_stream_id(&self) -> StreamId {
        self.stream_id
    }

    /// Returns a `FrameHeader` based on the current state of the `Frame`.
    fn get_header(&self) -> FrameHeader {
        (self.payload_len(), 0x1, self.flags.0, self.stream_id)
    }
}

impl FrameIR for HeadersFrame {
    fn serialize_into<B: FrameBuilder>(self, b: &mut B) -> io::Result<()> {
        b.write_header(self.get_header())?;
        let padded = self.is_set(HeadersFlag::Padded);
        if padded {
            b.write_all(&[self.padding_len.unwrap_or(0)])?;
        }
        // The stream dependency fields follow, if the priority flag is set
        if self.is_set(HeadersFlag::Priority) {
            let dep_buf = match self.stream_dep {
                Some(ref dep) => dep.serialize(),
                None => panic!("Priority flag set, but no dependency information given"),
            };
            b.write_all(&dep_buf)?;
        }
        // Now the actual headers fragment
        b.write_all(&self.header_fragment)?;
        // Finally, add the trailing padding, if required
        if padded {
            b.write_padding(self.padding_len.unwrap_or(0))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{HeadersFrame, HeadersFlag, StreamDependency};
    use solicit::frame::tests::build_padded_frame_payload;
    use solicit::tests::common::{raw_frame_from_parts, serialize_frame};
    use solicit::frame::{pack_header, Frame};

    /// Tests that a stream dependency structure can be correctly parsed by the
    /// `StreamDependency::parse` method.
    #[test]
    fn test_parse_stream_dependency() {
        {
            let buf = [0, 0, 0, 1, 5];

            let dep = StreamDependency::parse(&buf);

            assert_eq!(dep.stream_id, 1);
            assert_eq!(dep.weight, 5);
            // This one was not exclusive!
            assert!(!dep.is_exclusive)
        }
        {
            // Most significant bit set => is exclusive!
            let buf = [128, 0, 0, 1, 5];

            let dep = StreamDependency::parse(&buf);

            assert_eq!(dep.stream_id, 1);
            assert_eq!(dep.weight, 5);
            // This one was indeed exclusive!
            assert!(dep.is_exclusive)
        }
        {
            // Most significant bit set => is exclusive!
            let buf = [255, 255, 255, 255, 5];

            let dep = StreamDependency::parse(&buf);

            assert_eq!(dep.stream_id, (1 << 31) - 1);
            assert_eq!(dep.weight, 5);
            // This one was indeed exclusive!
            assert!(dep.is_exclusive);
        }
        {
            let buf = [127, 255, 255, 255, 5];

            let dep = StreamDependency::parse(&buf);

            assert_eq!(dep.stream_id, (1 << 31) - 1);
            assert_eq!(dep.weight, 5);
            // This one was not exclusive!
            assert!(!dep.is_exclusive);
        }
    }

    /// Tests that a stream dependency structure can be correctly serialized by
    /// the `StreamDependency::serialize` method.
    #[test]
    fn test_serialize_stream_dependency() {
        {
            let buf = [0, 0, 0, 1, 5];
            let dep = StreamDependency::new(1, 5, false);

            assert_eq!(buf, dep.serialize());
        }
        {
            // Most significant bit set => is exclusive!
            let buf = [128, 0, 0, 1, 5];
            let dep = StreamDependency::new(1, 5, true);

            assert_eq!(buf, dep.serialize());
        }
        {
            // Most significant bit set => is exclusive!
            let buf = [255, 255, 255, 255, 5];
            let dep = StreamDependency::new((1 << 31) - 1, 5, true);

            assert_eq!(buf, dep.serialize());
        }
        {
            let buf = [127, 255, 255, 255, 5];
            let dep = StreamDependency::new((1 << 31) - 1, 5, false);

            assert_eq!(buf, dep.serialize());
        }
    }

    /// Tests that a simple HEADERS frame is correctly parsed. The frame does
    /// not contain any padding nor priority information.
    #[test]
    fn test_headers_frame_parse_simple() {
        let data = b"123";
        let payload = data.to_vec();
        let header = (payload.len() as u32, 0x1, 0, 1);

        let raw = raw_frame_from_parts(header.clone(), payload.to_vec());
        let frame: HeadersFrame = Frame::from_raw(&raw).unwrap();

        assert_eq!(frame.header_fragment(), &data[..]);
        assert_eq!(frame.flags.0, 0);
        assert_eq!(frame.get_stream_id(), 1);
        assert!(frame.stream_dep.is_none());
        assert!(frame.padding_len.is_none());
    }

    /// Tests that a HEADERS frame with padding is correctly parsed.
    #[test]
    fn test_headers_frame_parse_with_padding() {
        let data = b"123";
        let payload = build_padded_frame_payload(data, 6);
        let header = (payload.len() as u32, 0x1, 0x08, 1);

        let raw = raw_frame_from_parts(header.clone(), payload.to_vec());
        let frame: HeadersFrame = Frame::from_raw(&raw).unwrap();

        assert_eq!(frame.header_fragment(), &data[..]);
        assert_eq!(frame.flags.0, 8);
        assert_eq!(frame.get_stream_id(), 1);
        assert!(frame.stream_dep.is_none());
        assert_eq!(frame.padding_len.unwrap(), 6);
    }

    /// Tests that a HEADERS frame with the priority flag (and necessary fields)
    /// is correctly parsed.
    #[test]
    fn test_headers_frame_parse_with_priority() {
        let data = b"123";
        let dep = StreamDependency::new(0, 5, true);
        let payload = {
            let mut buf: Vec<u8> = Vec::new();
            buf.extend(dep.serialize().to_vec().into_iter());
            buf.extend(data.to_vec().into_iter());

            buf
        };
        let header = (payload.len() as u32, 0x1, 0x20, 1);

        let raw = raw_frame_from_parts(header.clone(), payload.to_vec());
        let frame: HeadersFrame = Frame::from_raw(&raw).unwrap();

        assert_eq!(frame.header_fragment(), &data[..]);
        assert_eq!(frame.flags.0, 0x20);
        assert_eq!(frame.get_stream_id(), 1);
        assert_eq!(frame.stream_dep.unwrap(), dep);
        assert!(frame.padding_len.is_none());
    }

    /// Tests that a HEADERS frame with both padding and priority gets
    /// correctly parsed.
    #[test]
    fn test_headers_frame_parse_padding_and_priority() {
        let data = b"123";
        let dep = StreamDependency::new(0, 5, true);
        let full = {
            let mut buf: Vec<u8> = Vec::new();
            buf.extend(dep.serialize().to_vec().into_iter());
            buf.extend(data.to_vec().into_iter());

            buf
        };
        let payload = build_padded_frame_payload(&full, 4);
        let header = (payload.len() as u32, 0x1, 0x20 | 0x8, 1);

        let raw = raw_frame_from_parts(header.clone(), payload.to_vec());
        let frame: HeadersFrame = Frame::from_raw(&raw).unwrap();

        assert_eq!(frame.header_fragment(), &data[..]);
        assert_eq!(frame.flags.0, 0x20 | 0x8);
        assert_eq!(frame.get_stream_id(), 1);
        assert_eq!(frame.stream_dep.unwrap(), dep);
        assert_eq!(frame.padding_len.unwrap(), 4);
    }

    /// Tests that a HEADERS with stream ID 0 is considered invalid.
    #[test]
    fn test_headers_frame_parse_invalid_stream_id() {
        let data = b"123";
        let payload = data.to_vec();
        let header = (payload.len() as u32, 0x1, 0, 0);

        let raw = raw_frame_from_parts(header, payload);
        let frame: Option<HeadersFrame> = Frame::from_raw(&raw);

        assert!(frame.is_none());
    }

    /// Tests that the `HeadersFrame::parse` method considers any frame with
    /// a frame ID other than 1 in the frame header invalid.
    #[test]
    fn test_headers_frame_parse_invalid_type() {
        let data = b"123";
        let payload = data.to_vec();
        let header = (payload.len() as u32, 0x2, 0, 1);

        let raw = raw_frame_from_parts(header, payload);
        let frame: Option<HeadersFrame> = Frame::from_raw(&raw);

        assert!(frame.is_none());
    }

    /// Tests that a simple HEADERS frame (no padding, no priority) gets
    /// correctly serialized.
    #[test]
    fn test_headers_frame_serialize_simple() {
        let data = b"123";
        let payload = data.to_vec();
        let header = (payload.len() as u32, 0x1, 0, 1);
        let expected = {
            let headers = pack_header(&header);
            let mut res: Vec<u8> = Vec::new();
            res.extend(headers.to_vec().into_iter());
            res.extend(payload.into_iter());

            res
        };
        let frame = HeadersFrame::new(data.to_vec(), 1);

        let actual = serialize_frame(&frame);

        assert_eq!(expected, actual);
    }

    /// Tests that a HEADERS frame with padding is correctly serialized.
    #[test]
    fn test_headers_frame_serialize_with_padding() {
        let data = b"123";
        let payload = build_padded_frame_payload(data, 6);
        let header = (payload.len() as u32, 0x1, 0x08, 1);
        let expected = {
            let headers = pack_header(&header);
            let mut res: Vec<u8> = Vec::new();
            res.extend(headers.to_vec().into_iter());
            res.extend(payload.into_iter());

            res
        };
        let mut frame = HeadersFrame::new(data.to_vec(), 1);
        frame.set_padding(6);

        let actual = serialize_frame(&frame);

        assert_eq!(expected, actual);
    }

    /// Tests that a HEADERS frame with priority gets correctly serialized.
    #[test]
    fn test_headers_frame_serialize_with_priority() {
        let data = b"123";
        let dep = StreamDependency::new(0, 5, true);
        let payload = {
            let mut buf: Vec<u8> = Vec::new();
            buf.extend(dep.serialize().to_vec().into_iter());
            buf.extend(data.to_vec().into_iter());

            buf
        };
        let header = (payload.len() as u32, 0x1, 0x20, 1);
        let expected = {
            let headers = pack_header(&header);
            let mut res: Vec<u8> = Vec::new();
            res.extend(headers.to_vec().into_iter());
            res.extend(payload.into_iter());

            res
        };
        let frame = HeadersFrame::with_dependency(data.to_vec(), 1, dep.clone());

        let actual = serialize_frame(&frame);

        assert_eq!(expected, actual);
    }

    /// Tests that a HEADERS frame with both padding and a priority gets correctly
    /// serialized.
    #[test]
    fn test_headers_frame_serialize_padding_and_priority() {
        let data = b"123";
        let dep = StreamDependency::new(0, 5, true);
        let full = {
            let mut buf: Vec<u8> = Vec::new();
            buf.extend(dep.serialize().to_vec().into_iter());
            buf.extend(data.to_vec().into_iter());

            buf
        };
        let payload = build_padded_frame_payload(&full, 4);
        let header = (payload.len() as u32, 0x1, 0x20 | 0x8, 1);
        let expected = {
            let headers = pack_header(&header);
            let mut res: Vec<u8> = Vec::new();
            res.extend(headers.to_vec().into_iter());
            res.extend(payload.into_iter());

            res
        };
        let mut frame = HeadersFrame::with_dependency(data.to_vec(), 1, dep.clone());
        frame.set_padding(4);

        let actual = serialize_frame(&frame);

        assert_eq!(expected, actual);
    }

    /// Tests that the `HeadersFrame::is_headers_end` method returns the correct
    /// value depending on the `EndHeaders` flag being set or not.
    #[test]
    fn test_headers_frame_is_headers_end() {
        let mut frame = HeadersFrame::new(vec![], 1);
        assert!(!frame.is_headers_end());

        frame.set_flag(HeadersFlag::EndHeaders);
        assert!(frame.is_headers_end());
    }
}
