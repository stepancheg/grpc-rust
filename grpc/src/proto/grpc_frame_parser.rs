use crate::error;
use crate::proto::grpc_frame::parse_grpc_frame_header;
use crate::proto::grpc_frame::GRPC_HEADER_LEN;
use crate::result;
use bytes::Buf;
use bytes::Bytes;
use httpbis::BufGetBytes;
use httpbis::BytesDeque;

#[derive(Default, Debug)]
pub(crate) struct GrpcFrameParser {
    /// Next frame length; stripped prefix from the queue
    next_frame_len: Option<u32>,
    /// Unparsed bytes
    buffer: BytesDeque,
}

impl GrpcFrameParser {
    /// Add `Bytes` object to the queue.
    pub fn enqueue(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
    }

    /// Try populating `next_frame_len` field from the queue.
    fn fill_next_frame_len(&mut self) -> result::Result<Option<u32>> {
        if let None = self.next_frame_len {
            self.next_frame_len = parse_grpc_frame_header(&mut self.buffer)?;
        }
        Ok(self.next_frame_len)
    }

    /// Parse next frame if buffer has full frame.
    pub fn next_frame(&mut self) -> result::Result<Option<(Bytes, usize)>> {
        if let Some(len) = self.fill_next_frame_len()? {
            if self.buffer.remaining() >= len as usize {
                self.next_frame_len = None;
                return Ok(Some((
                    BufGetBytes::get_bytes(&mut self.buffer, len as usize),
                    len as usize + GRPC_HEADER_LEN,
                )));
            }
        }
        Ok(None)
    }

    /// Parse all frames from buffer.
    pub fn next_frames(&mut self) -> result::Result<(Vec<Bytes>, usize)> {
        let mut r = Vec::new();
        let mut consumed = 0;
        while let Some((frame, frame_consumed)) = self.next_frame()? {
            r.push(frame);
            consumed += frame_consumed;
        }
        Ok((r, consumed))
    }

    /// Buffered data is empty.
    pub fn is_empty(&self) -> bool {
        self.next_frame_len.is_none() && !self.buffer.has_remaining()
    }

    pub fn check_empty(&self) -> result::Result<()> {
        if !self.is_empty() {
            return Err(error::Error::Other("partial frame"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test() {
        fn parse(data: &[u8]) -> result::Result<Option<(Bytes, usize)>> {
            let mut parser = GrpcFrameParser::default();
            parser.enqueue(Bytes::copy_from_slice(data));
            parser.next_frame()
        }

        assert_eq!(None, parse(b"").unwrap());
        assert_eq!(None, parse(b"1").unwrap());
        assert_eq!(None, parse(b"14sc").unwrap());
        assert_eq!(None, parse(b"\x00\x00\x00\x00\x07\x0a\x05wo").unwrap());
        assert_eq!(
            Some((Bytes::copy_from_slice(b"\x0a\x05world"), 12)),
            parse(b"\x00\x00\x00\x00\x07\x0a\x05world").unwrap()
        );
    }
}
