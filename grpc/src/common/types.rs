use common::http_sink::HttpSink;
use common::sink::SinkUntyped;

pub(crate) trait Types: Send + Sync + 'static {
    type HttpSink: HttpSink;
    type SinkUntyped: SinkUntyped;
}
