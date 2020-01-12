use crate::common::http_sink::HttpSink;
use crate::common::sink::SinkUntyped;

pub(crate) trait Types: Send + Sync + 'static {
    type HttpSink: HttpSink;
    type SinkUntyped: SinkUntyped;
}
