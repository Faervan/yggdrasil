use tokio::{io::AsyncWriteExt, net::TcpListener, runtime::Runtime};
use yserde_derive::Package;

use super::*;

#[derive(Package, Debug)]
struct GameHello;
#[derive(Package, Default, Debug)]
struct GameMessage {
    message: String,
    sender: u16
}

#[test]
fn it_works() {
    let packages = PackageIndex::new(vec![
        Box::new(GameHello),
        Box::new(GameHello),
        Box::new(GameMessage::default()),
    ]);

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:9983").await.unwrap();
        let mut client = TcpStream::connect("127.0.0.1:9983").await.unwrap();
        let (mut receiver, _) = listener.accept().await.unwrap();
        println!("new api: as_bytes: {:?}", GameMessage {sender: 7, message: "teststring".to_string()}.as_bytes_indexed(&packages));
        client.write(&packages.pkg_as_bytes(Box::new(GameHello))).await.unwrap();
        client.write(&packages.pkg_as_bytes(Box::new(GameMessage {sender: 7, message: "teststring".to_string()}))).await.unwrap();
        let mut i = 0;
        loop {
            i += 1;
            match_pkg!(
                packages.read_async_tcp(&mut receiver).await.unwrap(),
                GameHello => |hello| {
                    println!("iteration {i}");
                    println!("hello! {hello:?}");
                    println!("as bytes: {:?}", packages.pkg_as_bytes(hello));
                },
                GameMessage => |msg| {
                    println!("iteration {i}");
                    println!("message! {msg:?}");
                    println!("as bytes: {:?}", packages.pkg_as_bytes(msg));
                }
            );
            if i == 2 {break;}
        }
    });
    assert!(true);
}
