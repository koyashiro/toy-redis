mod redis;

use anyhow::Result;

use redis::RedisServer;

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = RedisServer::bind("0.0.0.0:6379").await?;
    server.run().await?;
    Ok(())
}
