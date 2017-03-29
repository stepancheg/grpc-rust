use std::time::Duration;

#[derive(Default, Debug, Clone)]
pub struct HttpClientConf {
    /// TCP_NODELAY
    pub no_delay: Option<bool>,
    pub thread_name: Option<String>,
    pub connection_timeout: Option<Duration>
}
