use std::io;

pub fn io_error_other(desc: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, desc)
}
