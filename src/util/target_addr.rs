use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

/// A description of a connection target.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetAddr {
    /// Connect to an IP address.
    Ip(SocketAddr),
    /// Connect to a fully qualified domain name.
    ///
    /// The domain name will be passed along to the proxy server and DNS lookup
    /// will happen there.
    Domain(String, u16),
}

/// A trait for objects that can be converted to `TargetAddr`.
/// This trait is first used `.to_target_addr().context("Can't convert address to TargetAddr format")?;`
/// 注意这个映射关系 (str,u16)->(Ipv4Addr, u16)->SocketAddrV4->SocketAddr->TargetAddr::Ip
pub trait ToTargetAddr {
    /// Converts the value of `self` to a `TargetAddr`.
    fn to_target_addr(&self) -> io::Result<TargetAddr>;
}

impl<'a> ToTargetAddr for (&'a str, u16) {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        // try to parse as an IP first
        if let Ok(addr) = self.0.parse::<Ipv4Addr>() {
            return (addr, self.1).to_target_addr();
        }

        Ok(TargetAddr::Domain(self.0.to_owned(), self.1))
    }
}

impl ToTargetAddr for (Ipv4Addr, u16) {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        SocketAddrV4::new(self.0, self.1).to_target_addr()
    }
}

impl ToTargetAddr for SocketAddrV4 {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        SocketAddr::V4(*self).to_target_addr()
    }
}

impl ToTargetAddr for SocketAddr {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        Ok(TargetAddr::Ip(*self))
    }
}
