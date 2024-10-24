use rcon_server::listen;
use tokio::{sync::mpsc::unbounded_channel, task};

#[tokio::main(flavor = "current_thread")]
async fn main() -> tokio::io::Result<()> {
    let (s, mut r) = unbounded_channel();
    task::spawn(listen(None, "abc", s));
    loop {
        if let Some((sx, value)) = r.recv().await {
            let _ = sx.send(match value.as_str() {
                "ping" => "pong".to_string(),
                _ => format!("'{value}' is not a valid command, try 'ping' instead")
            });
        }
    }
}
