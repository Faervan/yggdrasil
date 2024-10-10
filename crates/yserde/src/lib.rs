//! Little library that intends to make it a bit easier to send and
//! receive structs via byte streams.
//!
//! Currently all data is expected to be send via [tokio's TcpStream](tokio::net::TcpStream)
//! or [tokio's UdpSocket](tokio::net::UdpSocket)
//!
//! ### Usage
//!
//! The Package trait is needed for all data that should be send:
//! ```rust
//! use yserde::Package;
//! use tokio::net::TcpStream;
//! use std::any::Any;
//!
//! #[derive(Debug)]
//! struct HelloPackage;
//! impl Package for HelloPackage {
//!     fn kind(&self) -> &'static str {
//!         "HelloPackage"
//!     }
//!     fn get_new(&self) -> Box<dyn Package> {
//!         Box::new(HelloPackage)
//!     }
//!     fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
//!         Ok(Box::new(HelloPackage))
//!     }
//! }
//! ```
use std::{any::Any, collections::HashMap, fmt::Debug};

use tokio::{io::AsyncReadExt, net::TcpStream};

pub trait Package: Any + Debug {
    fn kind(&self) -> &'static str;
    fn get_new(&self) -> Box<dyn Package>;
    fn as_bytes(&self) -> Vec<u8> {vec![]}
    fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>>;
}

pub struct PackageKinds {
    ser_map: HashMap<&'static str, u8>,
    de_map: HashMap<u8, Box<dyn Package>>,
}

impl PackageKinds {
    pub fn new(list: Vec<(&'static str, Box<dyn Package>)>) -> PackageKinds {
       let (ser_map, de_map) = list
                                .into_iter()
                                .enumerate()
                                .map(|(index, (string, obj))| ((string, index as u8), (index as u8, obj)))
                                .unzip();
        PackageKinds {
            de_map,
            ser_map,
        }
    }
    pub fn pkg_as_bytes<P: Package>(&self, package: P) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(*self.ser_map.get(package.kind()).expect("Package.kind() did not match any registered Packages"));
        bytes.extend_from_slice(&package.as_bytes());
        bytes
    }
    pub async fn get_from_tcp(&self, tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
        let mut kind_buf = [0;1];
        tcp.read(&mut kind_buf).await?;
        if let Some(obj) = self.de_map.get(&kind_buf[0]) {
            let obj = obj.get_new();
            tcp.readable().await?;
            return obj.from_bytes(tcp);
        }
        Err(tokio::io::ErrorKind::NotFound.into())
    }
}

#[macro_export]
macro_rules! match_pkg {
    ( $pkg:expr, $( $pkg_variant:ty => $handler:expr ), * ) => {
        'inner: {
            let pkg: Box<dyn Any> = $pkg;
            $(
                if pkg.is::<$pkg_variant>() {
                    $handler(pkg.downcast::<$pkg_variant>().unwrap());
                    break 'inner;
                }
            )*
        }
    };
}

#[cfg(test)]
mod tests {
    use tokio::{io::AsyncWriteExt, net::TcpListener, runtime::Runtime};

    use super::*;

    #[derive(Debug)]
    struct GameHello;
    #[derive(Default, Debug)]
    struct GameMessage {message: String, sender: u16}

    impl Package for GameHello {
        fn kind(&self) -> &'static str {
            "GameHello"
        }
        fn get_new(&self) -> Box<dyn Package> {
            Box::new(GameHello)
        }
        fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
            Ok(Box::new(GameHello))
        }
    }
    impl Package for GameMessage {
        fn kind(&self) -> &'static str {
            "GameMessage"
        }
        fn get_new(&self) -> Box<dyn Package> {
            Box::new(GameMessage {message: String::new(), sender: 0})
        }
        fn as_bytes(&self) -> Vec<u8> {
            let mut bytes = vec![];
            bytes.extend_from_slice(&self.sender.to_ne_bytes());
            bytes.push(self.message.len() as u8);
            bytes.extend_from_slice(self.message.as_bytes());
            bytes
        }
        fn from_bytes(&self, _tcp: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
            let mut sender_len_buf = [0;3];
            let _ = _tcp.try_read(&mut sender_len_buf);
            let mut message_buf: Vec<u8> = vec![0; sender_len_buf[2].into()];
            let _ = _tcp.try_read(&mut message_buf);
            Ok(Box::new(GameMessage {
                sender: u16::from_ne_bytes(sender_len_buf[..2].try_into().unwrap()),
                message: String::from_utf8_lossy(&message_buf).to_string(),
            }))
        }
    }

    #[test]
    fn it_works() {
        let packages = PackageKinds::new(vec![
            ("GameHello", Box::new(GameHello)),
            ("GameMessage", Box::new(GameMessage::default()))
        ]);

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind("127.0.0.1:9983").await.unwrap();
            let mut client = TcpStream::connect("127.0.0.1:9983").await.unwrap();
            let (mut receiver, _) = listener.accept().await.unwrap();
            client.write(&packages.pkg_as_bytes(GameHello)).await.unwrap();
            client.write(&packages.pkg_as_bytes(GameMessage {sender: 7, message: "teststring".to_string()})).await.unwrap();
            let mut i = 0;
            loop {
                i += 1;
                match_pkg!(
                    packages.get_from_tcp(&mut receiver).await.unwrap(),
                    GameHello => |hello| {
                        println!("iteration {i}");
                        println!("hello! {hello:?}");
                    },
                    GameMessage => |msg| {
                        println!("iteration {i}");
                        println!("message! {msg:?}");
                    }
                );
                if i == 2 {break;}
            }
        });
        assert!(true);
    }
}
