use std::result;

use error::*;

pub type Result<T> = result::Result<T, Error>;

