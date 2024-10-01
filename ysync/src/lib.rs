//! This crate is used as networking library in yggrasil
use std::time::Duration;

use bevy_math::{Quat, Vec3};
use tokio::{io::AsyncWriteExt, net::TcpStream};

/// functions and trait imlementations for use with the client side
pub mod client;
/// functions and trait imlementations for use with the server side
pub mod server;

#[derive(Debug)]
enum PackageType {
    LobbyConnect,
    LobbyDisconnect,
    LobbyConnectionAccept,
    LobbyConnectionDeny,
    LobbyUpdate(LobbyUpdate),
    InvalidPackage,
}

#[derive(Debug)]
pub enum LobbyUpdate {
    Connect,
    Disconnect,
    ConnectionInterrupt,
    Reconnect,
    Message,
}

pub enum LobbyUpdateData {
    Connect(Client),
    Disconnect(u16),
    ConnectionInterrupt(u16),
    Reconnect(u16),
    Message {
        sender: u16,
        length: u8,
        content: String,
    },
}

impl LobbyUpdateData {
    async fn write(self, tcp: &mut TcpStream) -> tokio::io::Result<()> {
        tcp.writable().await?;
        let mut bytes: Vec<u8> = vec![];
        bytes.push(u8::from(PackageType::LobbyUpdate(LobbyUpdate::from(&self))));
        match self {
            LobbyUpdateData::Connect(client) => bytes.extend_from_slice(&Vec::from(client)),
            LobbyUpdateData::Disconnect(client_id) => bytes.extend_from_slice(&client_id.to_ne_bytes()),
            LobbyUpdateData::ConnectionInterrupt(client_id) => bytes.extend_from_slice(&client_id.to_ne_bytes()),
            LobbyUpdateData::Reconnect(client_id) => bytes.extend_from_slice(&client_id.to_ne_bytes()),
            _ => {}
        }
        tcp.write(bytes.as_slice()).await?;
        Ok(())
    }
}

impl From<&LobbyUpdateData> for LobbyUpdate {
    fn from(value: &LobbyUpdateData) -> Self {
        match value {
            LobbyUpdateData::Connect(_) => LobbyUpdate::Connect,
            LobbyUpdateData::Disconnect(_) => LobbyUpdate::Disconnect,
            LobbyUpdateData::ConnectionInterrupt(_) => LobbyUpdate::ConnectionInterrupt,
            LobbyUpdateData::Reconnect(_) => LobbyUpdate::Reconnect,
            LobbyUpdateData::Message {..} => LobbyUpdate::Message,
        }
    }
}

impl From<PackageType> for u8 {
    fn from(value: PackageType) -> Self {
        match value {
            PackageType::LobbyConnect => 0,
            PackageType::LobbyDisconnect => 1,
            PackageType::LobbyConnectionAccept => 2,
            PackageType::LobbyConnectionDeny => 3,
            PackageType::LobbyUpdate(update) => {
                let n = 4;
                match update {
                    LobbyUpdate::Connect => n,
                    LobbyUpdate::Disconnect => n + 1,
                    LobbyUpdate::ConnectionInterrupt => n + 2,
                    LobbyUpdate::Reconnect => n + 3,
                    LobbyUpdate::Message => n + 4,
                }
            }
            PackageType::InvalidPackage => 255,
        }
    }
}

impl From<u8> for PackageType {
    fn from(value: u8) -> Self {
        match value {
            0 => PackageType::LobbyConnect,
            1 => PackageType::LobbyDisconnect,
            2 => PackageType::LobbyConnectionAccept,
            3 => PackageType::LobbyConnectionDeny,
            // upcoming nicely readable piece of code assigns the LobbyUpdate variants to 4..9
            mut i if i >= 4 && i <= 8 => PackageType::LobbyUpdate({
                i = i - 4;
                match i {
                    0 => LobbyUpdate::Connect,
                    1 => LobbyUpdate::Disconnect,
                    2 => LobbyUpdate::ConnectionInterrupt,
                    3 => LobbyUpdate::Reconnect,
                    _ => LobbyUpdate::Message,
                }
            }),
            _ => PackageType::InvalidPackage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: u16,
    pub in_game: bool,
    pub status: ClientStatus,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ClientStatus {
    Idle(Duration),
    Active,
}

impl Client {
    pub fn new(name: String) -> Client {
        Client { client_id: 0, in_game: false, status: ClientStatus::Active, name }
    }
}

#[derive(Debug)]
pub struct Game {
    pub game_id: u16,
    pub host_id: u16,
    pub password: bool,
    //max. 20 clients per game
    pub client_count: u8,
    pub clients: Vec<u16>,
}

#[derive(Debug)]
pub struct Lobby {
    pub client_count: u16,
    pub game_count: u16,
    pub clients: Vec<Client>,
    pub games: Vec<Game>,
}

#[derive(Debug)]
struct LobbyConnectionAcceptResponse {
    client_id: u16,
    lobby: Lobby,
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
