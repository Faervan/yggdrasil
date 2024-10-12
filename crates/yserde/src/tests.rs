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

/*impl Package for GameMessage {
    fn get_new(&self) -> Box<dyn Package> {
        Box::new(GameMessage::default())
    }
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.sender.to_ne_bytes());
        bytes.push(self.message.len() as u8);
        bytes.extend_from_slice(self.message.as_bytes());
        bytes
    }
    fn from_bytes(&self, _socket: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
        let mut sender_len_buf = [0;3];
        _socket.try_read(&mut sender_len_buf)?;
        let mut message_buf: Vec<u8> = vec![0; sender_len_buf[2].into()];
        _socket.try_read(&mut message_buf)?;
        Ok(Box::new(GameMessage {
            sender: u16::from_ne_bytes(sender_len_buf[..2].try_into().unwrap()),
            message: String::from_utf8_lossy(&message_buf).to_string(),
        }))
    }
}*/

#[test]
fn it_works() {
    let packages = PackageMap::new(vec![
        Box::new(GameHello),
        Box::new(GameHello),
        Box::new(GameMessage::default()),
    ]);

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:9983").await.unwrap();
        let mut client = TcpStream::connect("127.0.0.1:9983").await.unwrap();
        let (mut receiver, _) = listener.accept().await.unwrap();
        client.write(&packages.pkg_as_bytes(Box::new(GameHello))).await.unwrap();
        client.write(&packages.pkg_as_bytes(Box::new(GameMessage {sender: 7, message: "teststring".to_string()}))).await.unwrap();
        let mut i = 0;
        loop {
            i += 1;
            match_pkg!(
                packages.get_from_socket(&mut receiver).await.unwrap(),
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
