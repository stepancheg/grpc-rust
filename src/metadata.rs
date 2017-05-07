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

#[derive(Debug)]
pub enum MetadataDecodeError {
    Base64(base64::DecodeError),
}

impl From<base64::DecodeError> for MetadataDecodeError {
    fn from(decode_error: base64::DecodeError) -> Self {
        MetadataDecodeError::Base64(decode_error)
    }
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

    fn from_header(header: Header) -> Result<Option<MetadataEntry>, MetadataDecodeError> {
        if header.name().starts_with(b":") {
            return Ok(None);
        }
        if header.name().starts_with(b"grpc-") {
            return Ok(None);
        }
        let key = MetadataKey {
            name: Chars::try_from(header.name).expect("utf-8")
        };
        let value = match key.is_bin() {
            true => Bytes::from(base64::decode(&header.value)?),
            false => header.value,
        };
        Ok(Some(MetadataEntry {
            key: key,
            value: value,
        }))
    }
}

#[derive(Default, Debug)]
pub struct GrpcMetadata {
    entries: Vec<MetadataEntry>,
}

impl GrpcMetadata {
    pub fn new() -> GrpcMetadata {
        Default::default()
    }

    pub fn from_headers(headers: Headers) -> Result<GrpcMetadata, MetadataDecodeError> {
        let mut r = GrpcMetadata::new();
        for h in headers.0 {
            if let Some(e) = MetadataEntry::from_header(h)? {
                r.entries.push(e);
            }
        }
        Ok(r)
    }

    pub fn into_headers(self) -> Headers {
        Headers(self.entries.into_iter().map(MetadataEntry::into_header).collect())
    }
}
