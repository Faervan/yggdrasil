use ysync::server::listen;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    listen("127.0.0.1:9983").await?;
    Ok(())
}
