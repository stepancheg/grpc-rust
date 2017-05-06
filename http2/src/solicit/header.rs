use std::str;
use std::str::FromStr;
use std::fmt;
use std::borrow::Cow;

use bytes::Bytes;

/// A convenience struct representing a part of a header (either the name or the value).
pub struct HeaderPart(Bytes);

impl fmt::Debug for HeaderPart {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl From<Vec<u8>> for HeaderPart {
    fn from(vec: Vec<u8>) -> HeaderPart {
        HeaderPart(Bytes::from(vec))
    }
}

impl From<Bytes> for HeaderPart {
    fn from(bytes: Bytes) -> HeaderPart {
        HeaderPart(bytes)
    }
}

impl<'a> From<&'a [u8]> for HeaderPart {
    fn from(buf: &'a [u8]) -> HeaderPart {
        HeaderPart(Bytes::from(buf))
    }
}

impl<'a> From<Cow<'a, [u8]>> for HeaderPart {
    fn from(cow: Cow<'a, [u8]>) -> HeaderPart {
        HeaderPart(Bytes::from(cow.into_owned()))
    }
}

macro_rules! from_static_size_array {
    ($N:expr) => (
        impl<'a> From<&'a [u8; $N]> for HeaderPart {
            fn from(buf: &'a [u8; $N]) -> HeaderPart {
                buf[..].into()
            }
        }
    );
}

macro_rules! impl_from_static_size_array {
    ($($N:expr,)+) => {
        $(
            from_static_size_array!($N);
        )+
    }
}

impl_from_static_size_array!(
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    23,
);

impl From<String> for HeaderPart {
    fn from(s: String) -> HeaderPart {
        From::from(s.into_bytes())
    }
}

impl<'a> From<&'a str> for HeaderPart {
    fn from(s: &'a str) -> HeaderPart {
        From::from(s.as_bytes())
    }
}

impl<'a> From<Cow<'a, str>> for HeaderPart {
    fn from(cow: Cow<'a, str>) -> HeaderPart {
        From::from(cow.into_owned())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Header {
    pub name: Bytes,
    pub value: Bytes,
}

impl Header {
    /// Creates a new `Header` with the given name and value.
    ///
    /// The name and value need to be convertible into a `HeaderPart`.
    pub fn new<N: Into<HeaderPart>, V: Into<HeaderPart>>(name: N,
                                                                 value: V)
                                                                 -> Header {
        Header {
            name: name.into().0,
            value: value.into().0,
        }
    }

    /// Return a borrowed representation of the `Header` name.
    pub fn name(&self) -> &[u8] {
        &self.name
    }
    /// Return a borrowed representation of the `Header` value.
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

impl<N: Into<HeaderPart>, V: Into<HeaderPart>> From<(N, V)> for Header {
    fn from(p: (N, V)) -> Header {
        Header::new(p.0, p.1)
    }
}

#[derive(Default,Debug)]
pub struct Headers(pub Vec<Header>);

impl Headers {
    pub fn new() -> Headers {
        Default::default()
    }

    pub fn ok_200() -> Headers {
        Headers(vec![
            Header::new(":status", "200"),
        ])
    }

    pub fn internal_error_500() -> Headers {
        Headers(vec![
            Header::new(":status", "500"),
        ])
    }

    pub fn get_opt<'a>(&'a self, name: &str) -> Option<&'a str> {
        self.0.iter()
            .find(|h| h.name() == name.as_bytes())
            .and_then(|h| str::from_utf8(h.value()).ok())
    }

    pub fn get<'a>(&'a self, name: &str) -> &'a str {
        self.get_opt(name).unwrap()
    }

    pub fn get_opt_parse<I : FromStr>(&self, name: &str) -> Option<I> {
        self.get_opt(name)
            .and_then(|h| h.parse().ok())
    }

    pub fn status(&self) -> u32 {
        self.get_opt_parse(":status").unwrap()
    }

    pub fn path(&self) -> &str {
        self.get(":path")
    }

    pub fn method(&self) -> &str {
        self.get(":method")
    }

    pub fn add(&mut self, name: &str, value: &str) {
        self.0.push(Header::new(name, value));
    }

    pub fn extend(&mut self, headers: Headers) {
        self.0.extend(headers.0);
    }

}
