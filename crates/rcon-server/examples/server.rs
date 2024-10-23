use rcon_server::listen;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (s, _) = unbounded_channel();
    let _ = listen(None, "abc", s).await;
}
