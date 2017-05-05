//! The module contains the implementation of HTTP/2 frames.

use std::io;
use std::mem;

use bytes::Bytes;

use solicit::StreamId;
use solicit::frame::flags::*;

/// A helper macro that unpacks a sequence of 4 bytes found in the buffer with
/// the given identifier, starting at the given offset, into the given integer
/// type. Obviously, the integer type should be able to support at least 4
/// bytes.
///
/// # Examples
///
/// ```rust
/// let buf: [u8; 4] = [0, 0, 0, 1];
/// assert_eq!(1u32, unpack_octets_4!(buf, 0, u32));
/// ```
#[macro_escape]
macro_rules! unpack_octets_4 {
    ($buf:expr, $offset:expr, $tip:ty) => (
        (($buf[$offset + 0] as $tip) << 24) |
        (($buf[$offset + 1] as $tip) << 16) |
        (($buf[$offset + 2] as $tip) <<  8) |
        (($buf[$offset + 3] as $tip) <<  0)
    );
}

/// Parse the next 4 octets in the given buffer, assuming they represent an HTTP/2 stream ID.
/// This means that the most significant bit of the first octet is ignored and the rest interpreted
/// as a network-endian 31-bit integer.
#[inline]
fn parse_stream_id(buf: &[u8]) -> u32 {
    let unpacked = unpack_octets_4!(buf, 0, u32);
    // Now clear the most significant bit, as that is reserved and MUST be ignored when received.
    unpacked & !0x80000000
}

pub const FRAME_HEADER_LEN: usize = 9;

pub mod builder;
pub mod data;
pub mod headers;
pub mod rst_stream;
pub mod settings;
pub mod goaway;
pub mod ping;
pub mod window_update;
mod flags;

pub use self::builder::FrameBuilder;

/// Rexports related to the `DATA` frame.
pub use self::data::{DataFlag, DataFrame};
/// Rexports related to the `HEADERS` frame.
pub use self::headers::{HeadersFlag, HeadersFrame};
pub use self::rst_stream::RstStreamFrame;
/// Rexports related to the `SETTINGS` frame.
pub use self::settings::{SettingsFlag, SettingsFrame, HttpSetting};
pub use self::goaway::GoawayFrame;
pub use self::ping::PingFrame;
pub use self::window_update::WindowUpdateFrame;

/// An alias for the 9-byte buffer that each HTTP/2 frame header must be stored
/// in.
pub type FrameHeaderBuffer = [u8; 9];
/// An alias for the 4-tuple representing the components of each HTTP/2 frame
/// header.
pub type FrameHeader = (u32, u8, u8, u32);

/// Deconstructs a `FrameHeader` into its corresponding 4 components,
/// represented as a 4-tuple: `(length, frame_type, flags, stream_id)`.
///
/// The frame `type` and `flags` components are returned as their original
/// octet representation, rather than reinterpreted.
pub fn unpack_header(header: &FrameHeaderBuffer) -> FrameHeader {
    let length: u32 = ((header[0] as u32) << 16) | ((header[1] as u32) << 8) |
                       (header[2] as u32);
    let frame_type = header[3];
    let flags = header[4];
    let stream_id = parse_stream_id(&header[5..]);

    (length, frame_type, flags, stream_id)
}

/// Constructs a buffer of 9 bytes that represents the given `FrameHeader`.
pub fn pack_header(header: &FrameHeader) -> FrameHeaderBuffer {
    let &(length, frame_type, flags, stream_id) = header;

    [(((length >> 16) & 0x000000FF) as u8),
     (((length >>  8) & 0x000000FF) as u8),
     (((length      ) & 0x000000FF) as u8),
     frame_type,
     flags,
     (((stream_id >> 24) & 0x000000FF) as u8),
     (((stream_id >> 16) & 0x000000FF) as u8),
     (((stream_id >>  8) & 0x000000FF) as u8),
     (((stream_id      ) & 0x000000FF) as u8)]
}

/// A helper function that parses the given payload, considering it padded.
///
/// This means that the first byte is the length of the padding with that many
/// 0 bytes expected to follow the actual payload.
///
/// # Returns
///
/// A slice of the given payload where the actual one is found and the length
/// of the padding.
///
/// If the padded payload is invalid (e.g. the length of the padding is equal
/// to the total length), returns `None`.
fn parse_padded_payload(payload: Bytes) -> Option<(Bytes, u8)> {
    if payload.len() == 0 {
        // We make sure not to index the payload before we're sure how
        // large the buffer is.
        // If this is the case, the frame is invalid as no padding
        // length can be extracted, even though the frame should be
        // padded.
        return None;
    }
    let pad_len = payload[0] as usize;
    if pad_len >= payload.len() {
        // This is invalid: the padding length MUST be less than the
        // total frame size.
        return None;
    }

    Some((payload.slice(1, payload.len() - pad_len), pad_len as u8))
}

/// A trait that types that are an intermediate representation of HTTP/2 frames should implement.
/// It allows us to generically serialize any intermediate representation into an on-the-wire
/// representation.
pub trait FrameIR {
    /// Write out the on-the-wire representation of the frame into the given `FrameBuilder`.
    fn serialize_into<B: FrameBuilder>(self, builder: &mut B) -> io::Result<()>;

    fn serialize_into_vec(self) -> Vec<u8>
        where Self : Sized
    {
        let mut buf = io::Cursor::new(Vec::with_capacity(16));
        self.serialize_into(&mut buf).expect("serialize_into");
        buf.into_inner()
    }
}

/// A trait that all HTTP/2 frame structs need to implement.
pub trait Frame: Sized {
    /// The type that represents the flags that the particular `Frame` can take.
    /// This makes sure that only valid `Flag`s are used with each `Frame`.
    type FlagType: Flag;

    /// Creates a new `Frame` from the given `RawFrame` (i.e. header and
    /// payload), if possible.
    ///
    /// # Returns
    ///
    /// `None` if a valid `Frame` cannot be constructed from the given
    /// `RawFrame`. Some reasons why this may happen is a wrong frame type in
    /// the header, a body that cannot be decoded according to the particular
    /// frame's rules, etc.
    ///
    /// Otherwise, returns a newly constructed `Frame`.
    fn from_raw(raw_frame: &RawFrame) -> Option<Self>;

    /// Tests if the given flag is set for the frame.
    fn is_set(&self, flag: Self::FlagType) -> bool;
    /// Returns the `StreamId` of the stream to which the frame is associated
    fn get_stream_id(&self) -> StreamId;
    /// Returns a `FrameHeader` based on the current state of the `Frame`.
    fn get_header(&self) -> FrameHeader;
}

/// A struct that defines the format of the raw HTTP/2 frame, i.e. the frame
/// as it is read from the wire.
///
/// This format is defined in section 4.1. of the HTTP/2 spec.
///
/// The `RawFrame` struct simply stores the raw components of an HTTP/2 frame:
/// its header and the payload as a sequence of bytes.
///
/// It does not try to interpret the payload bytes, nor do any validation in
/// terms of its validity based on the frame type given in the header.
/// It is simply a wrapper around the two parts of an HTTP/2 frame.
#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct RawFrame {
    /// The raw frame representation, including both the raw header representation
    /// (in the first 9 bytes), followed by the raw payload representation.
    pub raw_content: Bytes,
}

pub struct RawFrameRef<'a> {
    pub raw_content: &'a [u8],
}

impl RawFrame {
    /// Parses a `RawFrame` from the bytes starting at the beginning of the given buffer.
    ///
    /// Returns the `None` variant when it is not possible to parse a raw frame from the buffer
    /// (due to there not being enough bytes in the buffer). If the `RawFrame` is successfully
    /// parsed it returns the frame, borrowing a part of the original buffer. Therefore, this
    /// method makes no copies, nor does it perform any extra allocations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use httpbis::solicit::frame::RawFrame;
    ///
    /// let buf = b"123";
    /// // Not enough bytes for even the header of the frame
    /// assert!(RawFrame::parse(&buf[..]).is_none());
    /// ```
    ///
    /// ```rust
    /// use httpbis::solicit::frame::RawFrame;
    ///
    /// let buf = vec![0, 0, 1, 0, 0, 0, 0, 0, 0];
    /// // Full header, but not enough bytes for the payload
    /// assert!(RawFrame::parse(&buf[..]).is_none());
    /// ```
    ///
    /// ```rust
    /// use httpbis::solicit::frame::RawFrame;
    ///
    /// let buf = vec![0, 0, 1, 0, 0, 0, 0, 0, 0, 1];
    /// // A full frame is extracted, even if in this case the frame itself is not valid (a DATA
    /// // frame associated to stream 0 is not valid in HTTP/2!). This, however, is not the
    /// // responsibility of the RawFrame.
    /// let frame = RawFrame::parse(&buf[..]).unwrap();
    /// assert_eq!(frame.as_ref(), &buf[..]);
    /// ```
    pub fn parse<B : Into<Bytes>>(into_buf: B) -> Option<RawFrame> {
        // TODO(mlalic): This might allow an extra parameter that specifies the maximum frame
        //               payload length?

        let buf = into_buf.into();

        if buf.len() < 9 {
            return None;
        }
        let header = unpack_header(unsafe {
            assert!(buf.len() >= 9);
            // We just asserted that this transmute is safe.
            mem::transmute(buf.as_ptr())
        });

        let payload_len = header.0 as usize;
        if buf[9..].len() < payload_len {
            return None;
        }

        let raw = &buf[..9 + payload_len];
        Some(raw.into())
    }

    pub fn as_frame_ref(&self) -> RawFrameRef {
        RawFrameRef {
            raw_content: &self.raw_content,
        }
    }

    pub fn frame_type(&self) -> u8 {
        self.as_frame_ref().frame_type()
    }

    /// Returns the total length of the `RawFrame`, including both headers, as well as the entire
    /// payload.
    #[inline]
    pub fn len(&self) -> usize {
        self.raw_content.len()
    }

    /// Returns a `Vec` of bytes representing the serialized (on-the-wire)
    /// representation of this raw frame.
    pub fn serialize(&self) -> &Bytes {
        &self.raw_content
    }

    /// Returns a `FrameHeader` instance corresponding to the headers of the
    /// `RawFrame`.
    pub fn header(&self) -> FrameHeader {
        unpack_header(unsafe {
            assert!(self.raw_content.len() >= 9);
            // We just asserted that this transmute is safe.
            mem::transmute(self.raw_content.as_ptr())
        })
    }

    pub fn get_stream_id(&self) -> StreamId {
        self.header().3
    }

    /// Returns a slice representing the payload of the `RawFrame`.
    pub fn payload(&self) -> Bytes {
        self.raw_content.slice_from(9)
    }
}

impl<'a> RawFrameRef<'a> {
    pub fn frame_type(&self) -> u8 {
        self.raw_content[3]
    }
}

impl AsRef<[u8]> for RawFrame {
    fn as_ref(&self) -> &[u8] {
        self.raw_content.as_ref()
    }
}
/// Provide a conversion from a `Vec`.
///
/// This conversion is unchecked and could cause the resulting `RawFrame` to be an
/// invalid HTTP/2 frame.
impl From<Vec<u8>> for RawFrame {
    fn from(raw: Vec<u8>) -> RawFrame {
        RawFrame { raw_content: Bytes::from(raw) }
    }
}
impl<'a> From<&'a [u8]> for RawFrame {
    fn from(raw: &'a [u8]) -> RawFrame {
        RawFrame { raw_content: Bytes::from(raw) }
    }
}

/// `RawFrame`s can be serialized to an on-the-wire format.
impl FrameIR for RawFrame {
    fn serialize_into<B: FrameBuilder>(self, b: &mut B) -> io::Result<()> {
        b.write_header(self.header())?;
        b.write_all(&self.payload())
    }
}

#[cfg(test)]
mod tests {
    use super::{unpack_header, pack_header, RawFrame, FrameIR};
    use std::io;

    /// Tests that the `unpack_header` function correctly returns the
    /// components of HTTP/2 frame headers.
    #[test]
    fn test_unpack_header() {
        {
            let header = [0; 9];
            assert_eq!((0, 0, 0, 0), unpack_header(&header));
        }
        {
            let header = [0, 0, 1, 2, 3, 0, 0, 0, 4];
            assert_eq!((1, 2, 3, 4), unpack_header(&header));
        }
        {
            let header = [0, 0, 1, 200, 100, 0, 0, 0, 4];
            assert_eq!((1, 200, 100, 4), unpack_header(&header));
        }
        {
            let header = [0, 0, 1, 0, 0, 0, 0, 0, 0];
            assert_eq!((1, 0, 0, 0), unpack_header(&header));
        }
        {
            let header = [0, 1, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!((256, 0, 0, 0), unpack_header(&header));
        }
        {
            let header = [1, 0, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!((256 * 256, 0, 0, 0), unpack_header(&header));
        }
        {
            let header = [0, 0, 0, 0, 0, 0, 0, 0, 1];
            assert_eq!((0, 0, 0, 1), unpack_header(&header));
        }
        {
            let header = [0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0, 1];
            assert_eq!(((1 << 24) - 1, 0, 0, 1), unpack_header(&header));
        }
        {
            let header = [0xFF, 0xFF, 0xFF, 0, 0, 1, 1, 1, 1];
            assert_eq!(((1 << 24) - 1, 0, 0, 1 + (1 << 8) + (1 << 16) + (1 << 24)),
                       unpack_header(&header));
        }
        {
            // Ignores reserved bit within the stream id (the most significant bit)
            let header = [0, 0, 1, 0, 0, 0x80, 0, 0, 1];
            assert_eq!((1, 0, 0, 1), unpack_header(&header));
        }
    }

    /// Tests that the `pack_header` function correctly returns the buffer
    /// corresponding to components of HTTP/2 frame headers.
    #[test]
    fn test_pack_header() {
        {
            let header = [0; 9];
            assert_eq!(pack_header(&(0, 0, 0, 0)), header);
        }
        {
            let header = [0, 0, 1, 2, 3, 0, 0, 0, 4];
            assert_eq!(pack_header(&(1, 2, 3, 4)), header);
        }
        {
            let header = [0, 0, 1, 200, 100, 0, 0, 0, 4];
            assert_eq!(pack_header(&(1, 200, 100, 4)), header);
        }
        {
            let header = [0, 0, 1, 0, 0, 0, 0, 0, 0];
            assert_eq!(pack_header(&(1, 0, 0, 0)), header);
        }
        {
            let header = [0, 1, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!(pack_header(&(256, 0, 0, 0)), header);
        }
        {
            let header = [1, 0, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!(pack_header(&(256 * 256, 0, 0, 0)), header);
        }
        {
            let header = [0, 0, 0, 0, 0, 0, 0, 0, 1];
            assert_eq!(pack_header(&(0, 0, 0, 1)), header);
        }
        {
            let header = [0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0, 1];
            assert_eq!(pack_header(&((1 << 24) - 1, 0, 0, 1)), header);
        }
        {
            let header = [0xFF, 0xFF, 0xFF, 0, 0, 1, 1, 1, 1];
            let header_components = ((1 << 24) - 1, 0, 0, 1 + (1 << 8) + (1 << 16) + (1 << 24));
            assert_eq!(pack_header(&header_components), header);
        }
    }

    /// Builds a `Vec` containing the given data as a padded HTTP/2 frame.
    ///
    /// It first places the length of the padding, followed by the data,
    /// followed by `pad_len` zero bytes.
    pub fn build_padded_frame_payload(data: &[u8], pad_len: u8) -> Vec<u8> {
        let sz = 1 + data.len() + pad_len as usize;
        let mut payload: Vec<u8> = Vec::with_capacity(sz);
        payload.push(pad_len);
        payload.extend(data.to_vec().into_iter());
        for _ in 0..pad_len {
            payload.push(0);
        }

        payload
    }

    /// Tests that a borrowed slice can be converted into a `RawFrame` due to the implementation of
    /// the `From<&'a [u8]>` trait.
    #[test]
    fn test_from_slice() {
        let buf = &b""[..];
        let frame = RawFrame::from(buf);
        assert_eq!(frame.as_ref(), buf);
    }

    /// Tests that the `RawFrame::serialize` method correctly serializes a
    /// `RawFrame`.
    #[test]
    fn test_raw_frame_serialize() {
        let data = b"123";
        let header = (data.len() as u32, 0x1, 0, 1);
        let buf = {
            let mut buf = Vec::new();
            buf.extend(pack_header(&header).to_vec().into_iter());
            buf.extend(data.to_vec().into_iter());
            buf
        };
        let raw: RawFrame = buf.clone().into();

        assert_eq!(raw.serialize().as_ref(), &buf[..]);
    }

    /// Tests that `RawFrame`s are correctly serialized to an on-the-wire format, when considered a
    /// `FrameIR` implementation.
    #[test]
    fn test_raw_frame_as_frame_ir() {
        let data = b"123";
        let header = (data.len() as u32, 0x1, 0, 1);
        let buf = {
            let mut buf = Vec::new();
            buf.extend(pack_header(&header).to_vec().into_iter());
            buf.extend(data.to_vec().into_iter());
            buf
        };
        let raw: RawFrame = buf.clone().into();

        let mut serialized = io::Cursor::new(Vec::new());
        raw.serialize_into(&mut serialized).unwrap();

        assert_eq!(serialized.into_inner(), buf);
    }

    /// Tests the `len` method of the `RawFrame`.
    #[test]
    fn test_raw_frame_len() {
        {
            // Not enough bytes for even the header of the frame
            let buf = b"123";
            let frame = RawFrame::from(&buf[..]);
            assert_eq!(buf.len(), frame.len());
        }
        {
            // Full header, but not enough bytes for the payload
            let buf = vec![0, 0, 1, 0, 0, 0, 0, 0, 0];
            let frame = RawFrame::from(&buf[..]);
            assert_eq!(buf.len(), frame.len());
        }
        {
            let buf = vec![0, 0, 1, 0, 0, 0, 0, 0, 0, 1];
            let frame = RawFrame::from(&buf[..]);
            assert_eq!(buf.len(), frame.len());
        }
    }
}
