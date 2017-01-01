use solicit::http::StaticHeader;

use http_common::*;


#[derive(Default)]
pub struct SimpleHttpMessage {
    pub headers: Vec<StaticHeader>,
    pub body: Vec<u8>,
}

impl SimpleHttpMessage {
    pub fn from_parts<I>(iter: I) -> SimpleHttpMessage
        where I : IntoIterator<Item=HttpStreamPart>
    {
        SimpleHttpMessage::from_part_content(iter.into_iter().map(|c| c.content))
    }

    pub fn from_part_content<I>(iter: I) -> SimpleHttpMessage
        where I : IntoIterator<Item=HttpStreamPartContent>
    {
        let mut r: SimpleHttpMessage = Default::default();
        for c in iter {
            match c {
                HttpStreamPartContent::Data(data) => {
                    r.body.extend(data);
                }
                HttpStreamPartContent::Headers(headers) => {
                    r.headers.extend(headers);
                }
            }
        }
        r
    }
}
