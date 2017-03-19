//! Defines the `FrameBuilder` trait and some default implementations of the trait.

use std::io;
use solicit::frame::{FrameHeader, pack_header};

/// A trait that provides additional methods for serializing HTTP/2 frames.
///
/// All methods have a default implementation in terms of the `io::Write` API, but types can
/// provide specialized more efficient implementations if possible. This is effectively a
/// workaround for specialization not existing yet in Rust.
pub trait FrameBuilder: io::Write + io::Seek {
    /// Write the given frame header as the next octets (i.e. without moving the cursor to the
    /// beginning of the buffer).
    fn write_header(&mut self, header: FrameHeader) -> io::Result<()> {
        self.write_all(&pack_header(&header))
    }

    /// Overwrite the previously written header, assuming it's the first byte sequence of the
    /// buffer.
    ///
    /// The default implementation seeks to the beginning of the buffer, writes the header, and
    /// then moves the cursor back to its previos position (i.e. offset from the beginning).
    fn overwrite_header(&mut self, header: FrameHeader) -> io::Result<()> {
        let current = self.seek(io::SeekFrom::Current(0))?;
        self.seek(io::SeekFrom::Start(0))?;
        self.write_header(header)?;
        self.seek(io::SeekFrom::Start(current))?;
        Ok(())
    }

    /// Copy all available bytes from the given `io::Read` instance.
    ///
    /// This method allows poor man's specialization for types that can implement the copy more
    /// efficiently than the `io::copy` function does (i.e. without the intermediate read into a
    /// stack-allocated buffer).
    fn copy_bytes_from<R: io::Read>(&mut self, provider: &mut R) -> io::Result<u64>
        where Self: Sized
    {
        io::copy(provider, self)
    }

    /// Write the given number of padding octets.
    ///
    /// The default implementation invokes the underlying Writer's `write` method `padding_length`
    /// times.
    ///
    /// Other `FrameBuilder` implementations could implement it more efficiently (e.g. if it is
    /// known that the `FrameBuilder` is backed by a zeroed buffer, there's no need to write
    /// anything, only increment a cursor/offset).
    fn write_padding(&mut self, padding_length: u8) -> io::Result<()> {
        for _ in 0..padding_length {
            self.write_all(&[0])?;
        }
        Ok(())
    }

    /// Write the given unsigned 32 bit integer to the underlying stream. The integer is written as
    /// four bytes in network endian style.
    fn write_u32(&mut self, num: u32) -> io::Result<()> {
        self.write_all(&[(((num >> 24) & 0x000000FF) as u8),
                         (((num >> 16) & 0x000000FF) as u8),
                         (((num >>  8) & 0x000000FF) as u8),
                         (((num      ) & 0x000000FF) as u8)])
    }
}

impl FrameBuilder for io::Cursor<Vec<u8>> {}

#[cfg(test)]
mod tests {
    use super::FrameBuilder;
    use std::io::{self, Write};

    use solicit::frame::pack_header;

    #[test]
    fn test_write_header_empty_buffer() {
        let mut buf = io::Cursor::new(Vec::new());
        let header = (10, 0x1, 0x0, 3);
        let expected = pack_header(&header);

        buf.write_header(header).unwrap();

        let frame = buf.into_inner();
        assert_eq!(frame, expected);
    }

    #[test]
    fn test_write_header_and_payload_empty() {
        let mut buf = io::Cursor::new(Vec::new());
        let header = (10, 0x1, 0x0, 3);
        let expected = {
            let mut buf = Vec::new();
            buf.extend(pack_header(&header).to_vec());
            buf.extend(vec![1, 2, 3, 4]);
            buf
        };

        buf.write_header(header).unwrap();
        buf.write_all(&[1, 2, 3, 4]).unwrap();

        let frame = buf.into_inner();
        assert_eq!(frame, expected);
    }

    #[test]
    fn test_rewrite_header_after_payload() {
        let mut buf = io::Cursor::new(Vec::new());
        let original_header = (10, 0x1, 0x0, 3);
        let actual_header = (5, 0x0, 0x0, 5);
        let expected = {
            let mut buf = Vec::new();
            buf.extend(pack_header(&actual_header).to_vec());
            buf.extend(vec![1, 2, 3, 4]);
            buf
        };

        // Sanity check for the test: the two headers must be different!
        assert!(original_header != actual_header);
        // First one header...
        buf.write_header(original_header).unwrap();
        // Then some payload...
        buf.write_all(&[1, 2, 3]).unwrap();
        // Now change the header!
        buf.overwrite_header(actual_header).unwrap();
        // ...and add some more data to the end
        buf.write_all(&[4]).unwrap();

        let frame = buf.into_inner();
        assert_eq!(frame, expected);
    }

    #[test]
    fn test_write_padding() {
        let mut buf = io::Cursor::new(Vec::new());
        buf.write_padding(5).unwrap();

        assert_eq!(buf.into_inner(), vec![0; 5]);
    }

    #[test]
    fn test_write_u32() {
        fn get_written(num: u32) -> Vec<u8> {
            let mut buf = io::Cursor::new(Vec::new());
            buf.write_u32(num).unwrap();
            buf.into_inner()
        }

        assert_eq!(get_written(0x0), vec![0, 0, 0, 0]);
        assert_eq!(get_written(0x1), vec![0, 0, 0, 1]);
        assert_eq!(get_written(0x10), vec![0, 0, 0, 0x10]);
        assert_eq!(get_written(0x10AB00CC), vec![0x10, 0xAB, 0, 0xCC]);
        assert_eq!(get_written(0xFFFFFFFF), vec![0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(get_written(0xEFFFFFFF), vec![0xEF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(get_written(0x7FFFFFFF), vec![0x7F, 0xFF, 0xFF, 0xFF]);
        assert_eq!(get_written(0xFFFFFF7F), vec![0xFF, 0xFF, 0xFF, 0x7F]);
    }
}
