
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