use ysync::client::ConnectionSocket;

fn main() {
    println!("Hello, world!");
    let x = ConnectionSocket::build("127.0.0.1:9983", "stk0".to_string());
    println!("{x:#?}");
}
