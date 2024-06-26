use anyhow::Context;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    consts, read_exact, util::{
        stream::tcp_connect,
        target_addr::{TargetAddr, ToTargetAddr},
    }, AuthenticationMethod, ReplyError, Result, Socks5Command, SocksError
};

use std::net::{SocketAddr, ToSocketAddrs};

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
        self.target_addr = Some(target_addr);

        // Request Lifecycle
        info!("Requesting headers `{:?}`...", &self.target_addr);
        self.request_header(cmd).await?;
        let bind_addr = self.read_request_reply().await?;

        Ok(bind_addr)
    }

    /// 3 Decide to whether or not, accept the authentication method.
    /// Don't forget that the methods list sent by the client, contains one or more methods.
    ///
    /// # Request
    /// ```test
    ///          +----+-----+-------+------+----------+----------+
    ///          |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
    ///          +----+-----+-------+------+----------+----------+
    ///          | 1  |  1  |   1   |  1   | Variable |    2     |
    ///          +----+-----+-------+------+----------+----------+
    /// ```
    ///
    /// # Help
    ///
    /// To debug request use a netcat server with hexadecimal output to parse the hidden bytes:
    ///
    /// ```bash
    ///    $ nc -k -l 80 | hexdump -C
    /// ```
    ///
    async fn request_header(&mut self, cmd: Socks5Command) -> Result<()> {
        let mut packet = [0u8; MAX_ADDR_LEN + 3];
        let padding: usize; // maximum len of the headers sent
                            // build our request packet with (socks version, Command, reserved)
        packet[..3].copy_from_slice(&[consts::SOCKS5_VERSION, cmd.as_u8(), 0x00]);

        match self.target_addr.as_ref() {
            None => {
                if cmd == Socks5Command::UDPAssociate {
                    debug!("UDPAssociate without target_addr, fallback to zeros.");
                    padding = 10;

                    packet[3] = 0x01;
                    packet[4..8].copy_from_slice(&[0, 0, 0, 0]); // ip
                    packet[8..padding].copy_from_slice(&[0, 0]); // port
                } else {
                    return Err(anyhow::Error::msg("target addr should be present").into());
                }
            }
            Some(target_addr) => match target_addr {
                TargetAddr::Ip(SocketAddr::V4(addr)) => {
                    debug!("TargetAddr::IpV4");
                    padding = 10;

                    packet[3] = 0x01;
                    debug!("addr ip {:?}", (*addr.ip()).octets());
                    packet[4..8].copy_from_slice(&(addr.ip()).octets()); // ip
                    packet[8..padding].copy_from_slice(&addr.port().to_be_bytes());
                    // port
                }
                TargetAddr::Ip(SocketAddr::V6(addr)) => {
                    return Err(anyhow::Error::msg("unsupported ipv6").into());
                }
                TargetAddr::Domain(ref domain, port) => {
                    debug!("TargetAddr::Domain");
                    if domain.len() > u8::MAX as usize {
                        return Err(SocksError::ExceededMaxDomainLen(domain.len()));
                    }
                    padding = 5 + domain.len() + 2;

                    packet[3] = 0x03; // Specify domain type
                    packet[4] = domain.len() as u8; // domain length
                    packet[5..(5 + domain.len())].copy_from_slice(domain.as_bytes()); // domain content
                    packet[(5 + domain.len())..padding].copy_from_slice(&port.to_be_bytes());
                    // port content (.to_be_bytes() convert from u16 to u8 type)
                }
            },
        }

        debug!("Bytes long version: {:?}", &packet[..]);
        debug!("Bytes shorted version: {:?}", &packet[..padding]);
        debug!("Padding: {}", &padding);

        // we limit the end of the packet right after the domain + port number, we don't need to print
        // useless 0 bytes, otherwise other protocol won't understand the request (like HTTP servers).
        self.socket
            .write(&packet[..padding])
            .await
            .context("Can't write request header's packet.")?;

        self.socket
            .flush()
            .await
            .context("Can't flush request header's packet")?;

        Ok(())
    }

    /// 4. The server send a confirmation (reply) that he had successfully connected (or not) to the
    /// remote server.
    async fn read_request_reply(&mut self) -> Result<TargetAddr> {
        let [version, reply, rsv, address_type] =
            read_exact!(self.socket, [0u8; 4]).context("Received malformed reply")?;
        debug!(
                "Reply received: [version: {version}, reply: {reply}, rsv: {rsv}, address_type: {address_type}]",
                version = version,
                reply = reply,
                rsv = rsv,
                address_type = address_type,
            );
        if version != consts::SOCKS5_VERSION {
            return Err(SocksError::UnsupportedSocksVersion(version));
        }

        if reply != consts::SOCKS5_REPLY_SUCCEEDED {
            return Err(ReplyError::from_u8(reply).into()); // Convert reply received into correct error
        }
        todo!()
    }
}

const MAX_ADDR_LEN: usize = 260;
