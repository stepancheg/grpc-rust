use crate::common::sink::SinkUntyped;

pub(crate) trait Types: Send + Sync + 'static {
    type SinkUntyped: SinkUntyped;
}
