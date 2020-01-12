use std::fmt;
use std::ops::Deref;
use std::str;

use bytes::Bytes;

#[derive(PartialEq, Eq, Clone)]
pub struct Chars(Bytes);

impl fmt::Debug for Chars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl Chars {
    pub fn from_static(s: &'static str) -> Chars {
        Chars(Bytes::from_static(s.as_bytes()))
    }

    pub fn into_inner(self) -> Bytes {
        self.0
    }

    pub fn try_from<B: Into<Bytes>>(b: B) -> Result<Chars, str::Utf8Error> {
        let bytes = b.into();
        str::from_utf8(&bytes)?;
        Ok(Chars(bytes))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn copy_from_str(s: &str) -> Chars {
        Chars(Bytes::copy_from_slice(s.as_bytes()))
    }
}

impl AsRef<str> for Chars {
    fn as_ref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl Deref for Chars {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<&'static str> for Chars {
    fn from(s: &'static str) -> Chars {
        Chars(Bytes::from(s))
    }
}

impl From<String> for Chars {
    fn from(s: String) -> Chars {
        Chars(Bytes::from(s))
    }
}
