#[derive(Default, Debug, Clone)]
pub struct HttpServerConf {
    /// TCP_NODELAY
    pub no_delay: Option<bool>,
    pub thread_name: Option<String>,
}
