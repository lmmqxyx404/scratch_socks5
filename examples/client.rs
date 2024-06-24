use socks5_scratch::Result;

#[derive(Debug)]
struct Opt {
    /// Socks5 server address + port. eg. `127.0.0.1:1080`
    pub socks_server: String,

    /// Target address server (not the socks server)
    pub target_addr: String,

    /// Target port server (not the socks server)
    pub target_port: u16,

    /// 可以不配置
    pub username: Option<String>,
    /// 可以不配置
    pub password: Option<String>,

    /// Don't perform the auth handshake, send directly the command request
    pub skip_auth: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    spawn_socks_client().await
}

async fn spawn_socks_client() -> Result<()> {
    let opt = Opt {
        password: None,
        username: None,
        skip_auth: true,
        socks_server: "127.0.0.1:1337".to_owned(),
        // 可以自行替换
        target_addr: "www.github.com".to_owned(),
        target_port: 443,
    };
    todo!()
}
