use std::collections::HashMap;

use bevy_math::{Quat, Vec3};

pub mod client;
pub mod server;

#[derive(Debug)]
enum PackageType {
    LobbyConnection,
    ConnectionAccept,
    ConnectionDeny,
    InvalidPackage,
}

impl From<PackageType> for u8 {
    fn from(value: PackageType) -> Self {
        match value {
            PackageType::LobbyConnection => 0,
            PackageType::ConnectionAccept => 1,
            PackageType::ConnectionDeny => 2,
            PackageType::InvalidPackage => 255,
        }
    }
}

impl From<u8> for PackageType {
    fn from(value: u8) -> Self {
        match value {
            0 => PackageType::LobbyConnection,
            1 => PackageType::ConnectionAccept,
            2 => PackageType::ConnectionDeny,
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
