use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocksError {

}

pub type Result<T, E = SocksError> = core::result::Result<T, E>;

pub mod client;
