use std::sync::mpsc::channel;
use ysync::client::ConnectionSocket;

fn main() {
    println!("Hello, world!");
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    let (s, r) = channel();
    rt.block_on(async move {
        let socket = ConnectionSocket::build("0.0.0.0:9983", "0.0.0.0:9983", "stk0".to_string()).await;
        println!("{socket:#?}");
        s.send(()).unwrap();
    });
    let _ = r.recv();
}
