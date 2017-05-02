use std::sync::Arc;

use native_tls::TlsAcceptor;


#[derive(Clone)]
pub enum ServerTlsOption {
    Plain,
    Tls(Arc<TlsAcceptor>),
}

