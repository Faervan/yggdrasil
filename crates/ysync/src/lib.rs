//! This crate is used as networking library in yggrasil
use std::time::Duration;

use bevy_math::{Quat, Vec3};
use bevy_utils::HashMap;
use tokio::{io::AsyncWriteExt, net::TcpStream};

/// functions and trait imlementations for use with the client side
pub mod client;
/// functions and trait imlementations for use with the server side
pub mod server;

#[cfg(test)]
mod tests;

#[derive(Debug)]
enum PackageType {
    LobbyConnect,
    LobbyDisconnect,
    LobbyConnectionAccept,
    LobbyConnectionDeny,
    GameCreation,
    GameDeletion,
    GameEntry,
    GameEntryRequest,
    GameEntryDenial,
    GameExit,
    GameWorld,
    LobbyUpdate(LobbyUpdate),
    GameUpdate(GameUpdate),
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

#[derive(Debug)]
pub enum GameUpdate {
    Creation,
    Deletion,
    Entry,
    Exit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameUpdateData {
    Creation(Game),
    Deletion(/*game_id*/u16),
    Entry {
        client_id: u16,
        game_id: u16,
    },
    Exit(/*client_id*/u16),
}

impl GameUpdateData {
    async fn write(self, tcp: &mut TcpStream) -> tokio::io::Result<()> {
        tcp.writable().await?;
        let mut bytes: Vec<u8> = vec![];
        bytes.push(u8::from(PackageType::GameUpdate(GameUpdate::from(&self))));
        match self {
            GameUpdateData::Creation(game) => bytes.extend_from_slice(&Vec::from(game)),
            GameUpdateData::Deletion(host_id) => bytes.extend_from_slice(&host_id.to_ne_bytes()),
            GameUpdateData::Entry { client_id, game_id } => {
                bytes.extend_from_slice(&client_id.to_ne_bytes());
                bytes.extend_from_slice(&game_id.to_ne_bytes());
            }
            GameUpdateData::Exit(client_id) => bytes.extend_from_slice(&client_id.to_ne_bytes()),
        }
        tcp.write(bytes.as_slice()).await?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
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
            LobbyUpdateData::Message { sender, length, content } => {
                bytes.extend_from_slice(&sender.to_ne_bytes());
                bytes.push(length);
                bytes.extend_from_slice(content.as_bytes());
            }
        }
        tcp.write(bytes.as_slice()).await?;
        Ok(())
    }
}

impl From<&GameUpdateData> for GameUpdate {
    fn from(value: &GameUpdateData) -> Self {
        match value {
            GameUpdateData::Creation(_) => GameUpdate::Creation,
            GameUpdateData::Deletion(_) => GameUpdate::Deletion,
            GameUpdateData::Entry {..} => GameUpdate::Entry,
            GameUpdateData::Exit(_) => GameUpdate::Exit,
        }
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
            PackageType::GameCreation => 4,
            PackageType::GameDeletion => 5,
            PackageType::GameEntry => 6,
            PackageType::GameEntryRequest => 7,
            PackageType::GameEntryDenial => 8,
            PackageType::GameExit => 9,
            PackageType::GameWorld => 10,
            PackageType::LobbyUpdate(update) => {
                let n = 11;
                match update {
                    LobbyUpdate::Connect => n,
                    LobbyUpdate::Disconnect => n + 1,
                    LobbyUpdate::ConnectionInterrupt => n + 2,
                    LobbyUpdate::Reconnect => n + 3,
                    LobbyUpdate::Message => n + 4,
                }
            }
            PackageType::GameUpdate(update) => {
                let n = 16;
                match update {
                    GameUpdate::Creation => n,
                    GameUpdate::Deletion => n + 1,
                    GameUpdate::Entry => n + 2,
                    GameUpdate::Exit => n + 3,
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
            4 => PackageType::GameCreation,
            5 => PackageType::GameDeletion,
            6 => PackageType::GameEntry,
            7 => PackageType::GameEntryRequest,
            8 => PackageType::GameEntryDenial,
            9 => PackageType::GameExit,
            10 => PackageType::GameWorld,
            11 => PackageType::LobbyUpdate(LobbyUpdate::Connect),
            12 => PackageType::LobbyUpdate(LobbyUpdate::Disconnect),
            13 => PackageType::LobbyUpdate(LobbyUpdate::ConnectionInterrupt),
            14 => PackageType::LobbyUpdate(LobbyUpdate::Reconnect),
            15 => PackageType::LobbyUpdate(LobbyUpdate::Message),
            16 => PackageType::GameUpdate(GameUpdate::Creation),
            17 => PackageType::GameUpdate(GameUpdate::Deletion),
            18 => PackageType::GameUpdate(GameUpdate::Entry),
            19 => PackageType::GameUpdate(GameUpdate::Exit),
            _ => PackageType::InvalidPackage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub client_id: u16,
    pub in_game: bool,
    pub status: ClientStatus,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Idle(Duration),
    Active,
}

impl Client {
    pub fn new(name: String) -> Client {
        Client { client_id: 0, in_game: false, status: ClientStatus::Active, name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub game_id: u16,
    pub host_id: u16,
    pub password: bool,
    pub game_name: String,
    pub clients: Vec<u16>,
}

#[derive(Debug)]
pub struct Lobby {
    pub client_count: u16,
    pub game_count: u16,
    pub clients: HashMap<u16, Client>,
    pub games: HashMap<u16, Game>,
}

#[derive(Debug)]
struct LobbyConnectionAcceptResponse {
    client_id: u16,
    lobby: Lobby,
}
