use std::{io::{Read, Write}, net::{TcpStream, ToSocketAddrs, UdpSocket}};

use bevy_math::{Quat, Vec3};

pub struct ConnectionSocket {
    //id of the game connected to
    game_id: u16,
    //id of the player
    sender_id: u16,
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
}

enum PackageType {
    LobbyConnection,
    ConnectionAccept,
    ConnectionDeny,
}

pub struct Client {
    pub client_id: u16,
    pub in_game: bool,
    pub name: String,
}

pub struct Game {
    pub game_id: u16,
    pub host_id: u16,
    pub password: bool,
    pub client_counts: u16,
    pub clients: Vec<Client>,
}

impl From<PackageType> for u8 {
    fn from(value: PackageType) -> Self {
        match value {
            PackageType::LobbyConnection => return 0,
        }
    }
}

impl ConnectionSocket {
    pub fn build<A: ToSocketAddrs>(lobby_addr: A, sender_name: String) -> std::io::Result<ConnectionSocket> {
        let mut tcp = TcpStream::connect(lobby_addr)?;
        let udp = UdpSocket::bind(lobby_addr)?;
        let mut package: Vec<u8> = vec![];
        package.push(u8::from(PackageType::LobbyConnection));
        package.extend_from_slice(sender_name.as_bytes());
        tcp.write(&package)?;
        let mut buf = [0; 20];
        tcp.read(&mut buf)?;
        Ok(ConnectionSocket {
            game_id,
            sender_id,
            tcp_stream: tcp,
            udp_socket: udp,
        })
    }
    pub fn build_package(self, package_builder)
    pub fn send(mut self, package: YPackage) -> std::io::Result<()> {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(&self.game_id.to_ne_bytes());
        bytes.extend_from_slice(&self.sender_id.to_ne_bytes());
        bytes.extend(&package.as_bytes());
        match package.package_type {
            PackageType::Connection | PackageType::Disconnection | PackageType::Message => {
                self.tcp_stream.write(&bytes)?;
            },
            PackageType::Movement | PackageType::Attack => {
                self.udp_socket.send(&bytes)?;
            },
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ypackage_en_and_decoding_works() {
        println!("\n------\n");
        let pkg = YPackage{
            package_type: PackageType::Message,
            sender: 12788,
            receiver: Some(23900),
            data: Some(Data::Message("hello world".to_string())),
        };
        let mut pkg_as_bytes: Vec<u8> = vec![1,3];
        pkg_as_bytes.extend(pkg.as_bytes());
        println!("pkg_as_bytes: {pkg_as_bytes:?}");
        assert_eq!(pkg, YPackage::from(pkg_as_bytes.as_slice()));
        println!("{pkg:?}");
        println!("\n------\n");
    }

    #[test]
    fn package_sending_works() {
        assert!(true);
    }
}
