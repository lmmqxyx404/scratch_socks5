use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::io::ErrorKind as IOErrorKind;

use crate::{ReplyError, Result};

pub async fn tcp_connect<T>(addr: T) -> Result<TcpStream>
where
    T: ToSocketAddrs,
{
    match TcpStream::connect(addr).await {
        Ok(o) => Ok(o),
        Err(e) => match e.kind() {
            // Match other TCP errors with ReplyError
            IOErrorKind::ConnectionRefused => Err(ReplyError::ConnectionRefused.into()),
            IOErrorKind::ConnectionAborted => Err(ReplyError::ConnectionNotAllowed.into()),
            IOErrorKind::ConnectionReset => Err(ReplyError::ConnectionNotAllowed.into()),
            IOErrorKind::NotConnected => Err(ReplyError::NetworkUnreachable.into()),
            _ => Err(e.into()), // #[error("General failure")] ?
        },
    }
}
