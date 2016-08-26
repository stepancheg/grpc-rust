use std::fmt;
use std::io;
use std::mem;

use solicit::http::Header;
use solicit::http::HttpResult;
use solicit::http::frame::RawFrame;
use solicit::http::frame::FrameIR;
use solicit::http::connection::ReceiveFrame;
use solicit::http::connection::HttpFrame;
use solicit::http::connection::SendFrame;

use misc::*;


#[allow(dead_code)]
pub struct HeaderDebug<'a>(pub &'a Header<'a, 'a>);

impl<'a> fmt::Debug for HeaderDebug<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
	    write!(fmt, "Header {{ name: {:?}, value: {:?} }}",
	    	BsDebug(self.0.name()), BsDebug(self.0.value()))
	}
}



pub struct OnceReceiveFrame<'a> {
    raw_frame: RawFrame<'a>,
    used: bool,
}

impl<'a> OnceReceiveFrame<'a> {
    pub fn new(raw_frame: RawFrame<'a>) -> OnceReceiveFrame<'a> {
        OnceReceiveFrame {
            raw_frame: raw_frame,
            used: false,
        }
    }
}

impl<'a> ReceiveFrame for OnceReceiveFrame<'a> {
    fn recv_frame(&mut self) -> HttpResult<HttpFrame> {
        assert!(!self.used);
        self.used = true;
        HttpFrame::from_raw(&self.raw_frame)
    }
}

pub struct VecSendFrame(pub Vec<u8>);

impl SendFrame for VecSendFrame {
    fn send_frame<F : FrameIR>(&mut self, frame: F) -> HttpResult<()> {
        let pos = self.0.len();
        let mut cursor = io::Cursor::new(mem::replace(&mut self.0, Vec::new()));
        cursor.set_position(pos as u64);
        try!(frame.serialize_into(&mut cursor));
        self.0 = cursor.into_inner();

        Ok(())
    }
}
