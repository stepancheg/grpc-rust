#[derive(Default, Debug, Clone)]
pub struct HttpClientConf {
    /// TCP_NODELAY
    pub no_delay: Option<bool>,
}
