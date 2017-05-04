use std::any::Any;
use std::fmt;

#[allow(dead_code)]
pub struct BsDebug<'a>(pub &'a [u8]);

impl<'a> fmt::Debug for BsDebug<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "b\"")?;
        let u8a: &[u8] = self.0;
        for &c in u8a {
            // ASCII printable
            if c >= 0x20 && c < 0x7f {
                write!(fmt, "{}", c as char)?;
            } else {
                write!(fmt, "\\x{:02x}", c)?;
            }
        }
        write!(fmt, "\"")?;
    	Ok(())
    }
}

pub fn any_to_string(any: Box<Any + Send + 'static>) -> String {
    if any.is::<String>() {
        *any.downcast::<String>().unwrap()
    } else if any.is::<&str>() {
        (*any.downcast::<&str>().unwrap()).to_owned()
    } else {
        "unknown any".to_owned()
    }
}
