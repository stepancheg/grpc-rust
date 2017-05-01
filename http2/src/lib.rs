#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_tls;
extern crate tokio_timer;

extern crate net2;
extern crate bytes;

pub mod solicit;

pub mod client_conf;
pub mod client_conn;
pub mod client;
pub mod server_conf;
pub mod server_conn;
pub mod server;
pub mod http_common;
pub mod message;

pub mod futures_misc;

mod tokio_oneshot;
pub mod assert_types;

pub mod hpack;
pub mod solicit_async;
pub mod solicit_misc;
pub mod bytesx;

mod misc;

pub use solicit::HttpScheme;
pub use solicit::HttpError;
pub use solicit::header::Header;
pub use solicit::header::Headers;

pub mod for_test {
    pub use server_conf::*;
    pub use server_conn::*;
    pub use server::*;
    pub use client_conf::*;
    pub use client_conn::*;
    pub use client::*;
    pub use http_common::*;
    pub use solicit_async::*;
    pub use futures_misc::*;
    pub use message::*;
    pub use solicit::header::*;
}
