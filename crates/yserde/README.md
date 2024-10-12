# YSerde
Little library that intends to make it a bit easier to send and
receive structs via byte streams.

Currently all data is expected to be send via [tokio's TcpStream](tokio::net::TcpStream)
or [tokio's UdpSocket](tokio::net::UdpSocket)

### Usage
To use this crate, you need to implement [Package] for the types you want to use
as packages and then create a [PackageMap] instance, which you can use to
convert your packages to and from bytes.

The [get_from_socket](PackageMap::get_from_socket) Method will return a `Box<dyn Any>`
which can be matched like an enum via the [match_pkg] macro.

### Example
```rust
use yserde::{Package, PackageMap, match_pkg};
use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}, runtime::Runtime};
use std::any::Any;

#[derive(Debug, Default, PartialEq)]
struct HelloPackage {
    some_string: String,
    some_num: u32,
}
impl Package for HelloPackage {
    fn get_new(&self) -> Box<dyn Package> {
        Box::new(HelloPackage::default())
    }
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.some_num.to_ne_bytes());
        bytes.push(self.some_string.len() as u8);
        bytes.extend_from_slice(self.some_string.as_bytes());
        bytes
    }
    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
        let mut buf = [0;5];
        _tcp.try_read(&mut buf)?;
        let mut string_buf = vec![0; buf[4].into()];
        _tcp.try_read(&mut string_buf)?;
        Ok(Box::new(HelloPackage {
            some_string: String::from_utf8_lossy(&string_buf).to_string(),
            some_num: u32::from_ne_bytes(buf[..4].try_into().unwrap())
        }))
    }
}

let tcp_packages = PackageMap::new(vec![
    Box::new(HelloPackage::default())
]);

let rt = Runtime::new().unwrap();
rt.block_on(async {
    let listener = TcpListener::bind("127.0.0.1:9983").await.unwrap();
    let mut tcp_client = TcpStream::connect("127.0.0.1:9983").await.unwrap();
    let (mut tcp_receiver, _) = listener.accept().await.unwrap();
    tcp_client.write(&tcp_packages.pkg_as_bytes(Box::new(HelloPackage {
        some_string: "This is a test String".to_string(),
        some_num: 267_509
    }))).await.unwrap();

    match_pkg!(
        tcp_packages.get_from_socket(&mut tcp_receiver).await.unwrap(),
        HelloPackage => |hello: Box<HelloPackage>| {
            println!("hello!\n{hello:#?}");
            assert_eq!(
                *hello,
                HelloPackage {
                    some_string: "This is a test String".to_string(),
                    some_num: 267_509
                }
            );
       }
    );
});
```
