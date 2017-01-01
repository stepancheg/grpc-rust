#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;

//extern crate solicit;
extern crate solicit_fork as solicit;
extern crate hpack;

pub mod client_conn;
pub mod client;
pub mod server_conn;
pub mod server;
pub mod http_common;

pub mod futures_misc;

mod tokio_oneshot;
pub mod assert_types;

pub mod solicit_async;
pub mod solicit_misc;

mod misc;

pub use solicit::http::HttpScheme;
pub use solicit::http::HttpError;
pub use solicit::http::Header;
pub use solicit::http::StaticHeader;

pub mod for_test {
    pub use server_conn::*;
    pub use client_conn::*;
    pub use http_common::*;
    pub use solicit_async::*;
    pub use futures_misc::*;
}
