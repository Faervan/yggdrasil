//! This crate is used as networking library in yggrasil
use std::time::Duration;

use bevy_math::{Quat, Vec3};

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
