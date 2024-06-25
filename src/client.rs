use anyhow::Context;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    util::{
        stream::tcp_connect,
        target_addr::{TargetAddr, ToTargetAddr},
    },
    AuthenticationMethod, Result, Socks5Command,
};

use std::net::ToSocketAddrs;

use tokio::net::TcpStream;

/// 客户端的一些基本设置
#[derive(Debug)]
pub struct Config {
    /// Timeout of the socket connect
    connect_timeout: Option<u64>,
    /// Avoid useless roundtrips if we don't need the Authentication layer
    /// make sure to also activate it on the server side.
    skip_auth: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            connect_timeout: None,
            skip_auth: false,
        }
    }
}

impl Config {
    /// How much time it should wait until the socket connect times out.
    pub fn set_connect_timeout(&mut self, n: u64) -> &mut Self {
        self.connect_timeout = Some(n);
        self
    }

    pub fn set_skip_auth(&mut self, value: bool) -> &mut Self {
        self.skip_auth = value;
        self
    }
}

/// A SOCKS5 client.
/// `Socks5Stream` implements [`AsyncRead`] and [`AsyncWrite`].
#[derive(Debug)]
pub struct Socks5Stream<S: AsyncRead + AsyncWrite + Unpin> {
    socket: S,
    target_addr: Option<TargetAddr>,
    config: Config,
}

/// Api if you want to use TcpStream to create a new connection to the SOCKS5 server.
impl Socks5Stream<TcpStream> {
    /// Connects to a target server through a SOCKS5 proxy.
    pub async fn connect<T>(
        socks_server: T,
        target_addr: String,
        target_port: u16,
        config: Config,
    ) -> Result<Self>
    where
        T: ToSocketAddrs,
    {
        Self::connect_raw(
            Socks5Command::TCPConnect,
            socks_server,
            target_addr,
            target_port,
            None,
            config,
        )
        .await
    }

    /// Process clients SOCKS requests
    /// This is the entry point where a whole request is processed.
    pub async fn connect_raw<T>(
        cmd: Socks5Command,
        socks_server: T,
        target_addr: String,
        target_port: u16,
        auth: Option<AuthenticationMethod>,
        config: Config,
    ) -> Result<Self>
    where
        T: ToSocketAddrs,
    {
        let addr = socks_server
            .to_socket_addrs()?
            .next()
            .context("unreachable")?;
        let socket = tcp_connect(addr).await?;
        info!("Connected @ {}", &socket.peer_addr()?);

        // Specify the target, here domain name, dns will be resolved on the server side
        let target_addr = (target_addr.as_str(), target_port)
            .to_target_addr()
            .context("Can't convert address to TargetAddr format")?;

        // upgrade the TcpStream to Socks5Stream
        let mut socks_stream = Self::use_stream(socket, auth, config).await?;
        socks_stream.request(cmd, target_addr).await?;

        Ok(socks_stream)
    }
}

impl<S> Socks5Stream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// 1 Possibility to use a stream already created rather than
    /// creating a whole new `TcpStream::connect()`.
    pub async fn use_stream(
        socket: S,
        auth: Option<AuthenticationMethod>,
        config: Config,
    ) -> Result<Self> {
        let mut stream = Socks5Stream {
            socket,
            config,
            target_addr: None,
        };

        // Auth none is always used by default.
        let mut methods = vec![AuthenticationMethod::None];

        if let Some(method) = auth {
            // add any other method if supplied
            methods.push(method);
        }

        // Handshake Lifecycle
        if !stream.config.skip_auth {
            debug!("to auth");
            // let methods = stream.send_version_and_methods(methods).await?;
            // stream.which_method_accepted(methods).await?;
        } else {
            debug!("skipping auth");
        }
        Ok(stream)
    }
    /// 2
    pub async fn request(
        &mut self,
        cmd: Socks5Command,
        target_addr: TargetAddr,
    ) -> Result<TargetAddr> {
        todo!()
    }
}
