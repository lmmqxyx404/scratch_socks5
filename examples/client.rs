use socks5_scratch::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    spawn_socks_client().await
}

async fn spawn_socks_client() -> Result<()> {
    todo!()
}
