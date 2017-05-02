use solicit::HttpScheme;

pub enum ClientTlsOption {
    Plain,
    Tls(String), // domain
}

impl ClientTlsOption {
    pub fn http_scheme(&self) -> HttpScheme {
        match self {
            &ClientTlsOption::Plain => HttpScheme::Http,
            &ClientTlsOption::Tls(..) => HttpScheme::Https,
        }
    }
}
