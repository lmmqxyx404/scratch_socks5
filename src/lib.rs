#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocksError {
    /// 1 used for transfer the std::io::Error to SocksError `.to_socket_addrs()?`
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    /// 2 used for transfer the anyhow::Error to SocksError `socks_server.to_socket_addrs()?`   #[error("Other: `{0}`.")]
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    /// 3 used for transfer the ReplyError to SocksError
    #[error("Error with reply: {0}.")]
    ReplyError(#[from] ReplyError),
    /// 4
    #[error("Domain exceeded max sequence length")]
    ExceededMaxDomainLen(usize),
    /// 5
    #[error("Unsupported SOCKS version `{0}`.")]
    UnsupportedSocksVersion(u8),
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

/// SOCKS5 reply code
#[derive(Error, Debug, Copy, Clone)]
pub enum ReplyError {
    #[error("Succeeded")]
    Succeeded,
    #[error("General failure")]
    GeneralFailure,
    #[error("Connection not allowed by ruleset")]
    ConnectionNotAllowed,
    #[error("Network unreachable")]
    NetworkUnreachable,
    #[error("Host unreachable")]
    HostUnreachable,
    #[error("Connection refused")]
    ConnectionRefused,
    #[error("Connection timeout")]
    ConnectionTimeout,
    #[error("TTL expired")]
    TtlExpired,
    #[error("Command not supported")]
    CommandNotSupported,
    #[error("Address type not supported")]
    AddressTypeNotSupported,
    //    OtherReply(u8),
}

#[rustfmt::skip]
pub mod consts {
    pub const SOCKS5_VERSION:                          u8 = 0x05;

    pub const SOCKS5_CMD_TCP_CONNECT:                  u8 = 0x01;
    pub const SOCKS5_CMD_TCP_BIND:                     u8 = 0x02;
    pub const SOCKS5_CMD_UDP_ASSOCIATE:                u8 = 0x03;

    pub const SOCKS5_REPLY_SUCCEEDED:                  u8 = 0x00;
}

#[allow(dead_code)]
impl Socks5Command {
    #[inline]
    #[rustfmt::skip]
    fn as_u8(&self) -> u8 {
        match self {
            Socks5Command::TCPConnect   => consts::SOCKS5_CMD_TCP_CONNECT,
            Socks5Command::TCPBind      => consts::SOCKS5_CMD_TCP_BIND,
            Socks5Command::UDPAssociate => consts::SOCKS5_CMD_UDP_ASSOCIATE,
        }
    }
}

impl ReplyError {
    #[inline]
    #[rustfmt::skip]
    pub fn from_u8(code: u8) -> ReplyError {
        match code {
            consts::SOCKS5_REPLY_SUCCEEDED                  => ReplyError::Succeeded,
            _                                               => unreachable!("ReplyError code unsupported."),
        }
    }
}
