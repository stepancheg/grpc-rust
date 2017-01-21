mod task_data;

mod stream_single;
mod stream_err;
mod stream_merge2;
mod stream_with_eof;
mod stream_with_eof_and_error;
pub mod stream_deferred;
pub mod stream_concat;
pub mod stream_concat3;

pub use self::task_data::TaskRcMut;

pub use self::stream_single::stream_single;

pub use self::stream_err::stream_err;
pub use self::stream_err::StreamErr;

pub use self::stream_merge2::stream_merge2;
pub use self::stream_merge2::StreamMerge2;
pub use self::stream_merge2::Merged2Item;

pub use self::stream_with_eof::*;
pub use self::stream_with_eof_and_error::*;

pub use self::stream_deferred::stream_deferred;
pub use self::stream_concat::stream_concat;
pub use self::stream_concat::Concat;
pub use self::stream_concat3::stream_concat3;
pub use self::stream_concat3::Concat3;
