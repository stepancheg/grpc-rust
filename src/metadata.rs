use base64;

use httpbis::solicit::header::*;

use bytes::Bytes;
use chars::Chars;

#[derive(Debug)]
pub struct MetadataKey {
    name: Chars,
}

impl MetadataKey {
    pub fn is_bin(&self) -> bool {
        self.name.ends_with("-bin")
    }
}

#[derive(Debug)]
struct MetadataEntry {
    key: MetadataKey,
    value: Bytes,
}

impl MetadataEntry {
    fn into_header(self) -> Header {
        let is_bin = self.key.is_bin();

        let value = match is_bin {
            true => Bytes::from(base64::encode(&self.value)),
            false => self.value,
        };

        Header::new(self.key.name.into_inner(), value)
    }
}

#[derive(Default, Debug)]
pub struct Metadata {
    entries: Vec<MetadataEntry>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Default::default()
    }

    pub fn into_headers(self) -> Headers {
        Headers(self.entries.into_iter().map(MetadataEntry::into_header).collect())
    }
}
