use yserde_bytes::Package;

#[derive(Package)]
struct TestStruct;

#[derive(Package)]
enum TestEnum {
    A,
    B,
    C
}

fn main() {
    println!("Hello, world!");
}
