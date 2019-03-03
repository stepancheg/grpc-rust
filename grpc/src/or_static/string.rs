use std::fmt;
use std::ops;

#[derive(Clone, Eq)]
pub enum StringOrStatic {
    String(String),
    Static(&'static str),
}

impl From<&str> for StringOrStatic {
    fn from(s: &str) -> Self {
        StringOrStatic::String(s.to_owned())
    }
}

impl From<String> for StringOrStatic {
    fn from(s: String) -> Self {
        StringOrStatic::String(s)
    }
}

impl StringOrStatic {
    pub fn as_str(&self) -> &str {
        match self {
            StringOrStatic::String(s) => &s,
            StringOrStatic::Static(s) => s,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl ops::Deref for StringOrStatic {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<StringOrStatic> for StringOrStatic {
    fn eq(&self, other: &StringOrStatic) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<str> for StringOrStatic {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for StringOrStatic {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<StringOrStatic> for str {
    fn eq(&self, other: &StringOrStatic) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<StringOrStatic> for &str {
    fn eq(&self, other: &StringOrStatic) -> bool {
        *self == other.as_str()
    }
}

impl fmt::Display for StringOrStatic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringOrStatic::String(s) => fmt::Display::fmt(s, f),
            StringOrStatic::Static(s) => fmt::Display::fmt(s, f),
        }
    }
}
