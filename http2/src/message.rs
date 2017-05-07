use bytes::Bytes;

use solicit::header::*;

use bytesx::*;

use http_common::*;


pub struct SimpleHttpMessage {
    pub headers: Headers,
    pub body: Bytes,
}

// TODO: https://github.com/carllerche/bytes/commit/37f6cabd96a6200b0b3cb1d743be9c0cf75d1085
impl Default for SimpleHttpMessage {
    fn default() -> Self {
        SimpleHttpMessage {
            headers: Default::default(),
            body: Bytes::new(),
        }
    }
}

impl SimpleHttpMessage {
    pub fn new() -> SimpleHttpMessage {
        Default::default()
    }

    pub fn from_parts<I>(iter: I) -> SimpleHttpMessage
        where I : IntoIterator<Item= HttpStreamPart>
    {
        SimpleHttpMessage::from_part_content(iter.into_iter().map(|c| c.content))
    }

    pub fn from_part_content<I>(iter: I) -> SimpleHttpMessage
        where I : IntoIterator<Item=HttpStreamPartContent>
    {
        let mut r: SimpleHttpMessage = Default::default();
        for c in iter {
            r.add(c);
        }
        r
    }

    pub fn add(&mut self, part: HttpStreamPartContent) {
        match part {
            HttpStreamPartContent::Headers(headers) => {
                self.headers.extend(headers);
            }
            HttpStreamPartContent::Data(data) => {
                bytes_extend_with(&mut self.body, data);
            }
        }
    }
}
