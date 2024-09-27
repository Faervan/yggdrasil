use ysync::client::ConnectionSocket;

fn main() {
    println!("Hello, world!");
    let x = ConnectionSocket::build("91.108.102.51:9983", "stk0".to_string());
    println!("{x:#?}");
}
