use std::result;

use crate::error::*;

pub type Result<T> = result::Result<T, Error>;
