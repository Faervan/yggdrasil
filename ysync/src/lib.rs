//! This crate is used as networking library in yggrasil
use std::collections::HashMap;

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
    InvalidPackage,
}

impl From<PackageType> for u8 {
    fn from(value: PackageType) -> Self {
        match value {
            PackageType::LobbyConnect => 0,
            PackageType::LobbyDisconnect => 1,
            PackageType::LobbyConnectionAccept => 2,
            PackageType::LobbyConnectionDeny => 3,
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
            _ => PackageType::InvalidPackage,
        }
    }
}

#[derive(Debug)]
pub struct Client {
    pub client_id: u16,
    pub in_game: bool,
    pub name: String,
}

impl Client {
    pub fn new(name: String) -> Client {
        Client { client_id: 0, in_game: false, name }
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
    pub clients: HashMap<u16, Client>,
    pub games: HashMap<u16, Game>,
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
