use ysync::client::ConnectionSocket;

fn main() {
    println!("Hello, world!");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        let socket = ConnectionSocket::build("0.0.0.0:9983", "0.0.0.0:9983", "stk0".to_string()).await;
        println!("{socket:#?}");
    });
}
