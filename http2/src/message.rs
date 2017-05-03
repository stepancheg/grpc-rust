use bytes::Bytes;

use solicit::header::*;

use bytesx::*;

use http_common::*;


pub struct SimpleHttpMessage {
    pub headers: Headers,
    pub body: Bytes,
}

impl Default for SimpleHttpMessage {
    fn default() -> Self {
        SimpleHttpMessage {
            headers: Default::default(),
            body: Bytes::new(),
        }
    }
}

impl SimpleHttpMessage {
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
            match c {
                HttpStreamPartContent::Headers(headers) => {
                    r.headers.extend(headers);
                }
                HttpStreamPartContent::Data(data) => {
                    bytes_extend_with(&mut r.body, data);
                }
            }
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
