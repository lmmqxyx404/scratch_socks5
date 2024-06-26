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

/// Easy to destructure bytes buffers by naming each fields:
///
/// # Examples (before)
///
/// ```ignore
/// let mut buf = [0u8; 2];
/// stream.read_exact(&mut buf).await?;
/// let [version, method_len] = buf;
///
/// assert_eq!(version, 0x05);
/// ```
///
/// # Examples (after)
///
/// ```ignore
/// let [version, method_len] = read_exact!(stream, [0u8; 2]);
///
/// assert_eq!(version, 0x05);
/// ```
#[macro_export]
macro_rules! read_exact {
    ($stream: expr, $array: expr) => {{
        let mut x = $array;
        //        $stream
        //            .read_exact(&mut x)
        //            .await
        //            .map_err(|_| io_err("lol"))?;
        $stream.read_exact(&mut x).await.map(|_| x)
    }};
}