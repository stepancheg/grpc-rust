use std::sync::Arc;

use native_tls::TlsConnector;

use solicit::HttpScheme;

pub enum ClientTlsOption {
    Plain,
    Tls(String, Arc<TlsConnector>), // domain
}

impl ClientTlsOption {
    pub fn http_scheme(&self) -> HttpScheme {
        match self {
            &ClientTlsOption::Plain => HttpScheme::Http,
            &ClientTlsOption::Tls(..) => HttpScheme::Https,
        }
    }
}
