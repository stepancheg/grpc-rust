#[derive(Default, Debug, Clone)]
pub struct HttpServerConf {
    /// TCP_NODELAY
    pub no_delay: Option<bool>,
    pub thread_name: Option<String>,
    /// Ignored on Windows
    pub reuse_port: Option<bool>,
    pub backlog: Option<i32>,
}
