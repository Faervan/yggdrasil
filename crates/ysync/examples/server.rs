use ysync::server::listen;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    listen("0.0.0.0:9983", Some((10010, "abc".to_string()))).await?;
    Ok(())
}
