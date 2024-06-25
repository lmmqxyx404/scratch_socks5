#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocksError {
    /// used for transfer the std::io::Error to SocksError `.to_socket_addrs()?`
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    /// used for transfer the anyhow::Error to SocksError `socks_server.to_socket_addrs()?`   #[error("Other: `{0}`.")]
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T, E = SocksError> = core::result::Result<T, E>;

/// 1
pub mod client;
/// 2
pub mod util;

#[derive(Debug, PartialEq)]
pub enum Socks5Command {
    TCPConnect,
    TCPBind,
    UDPAssociate,
}

#[derive(Debug, PartialEq)]
pub enum AuthenticationMethod {
    None,
    Password { username: String, password: String },
}
