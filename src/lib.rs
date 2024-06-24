#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocksError {

}

pub type Result<T, E = SocksError> = core::result::Result<T, E>;

/// 1
pub mod client;
/// 2
pub mod util;

