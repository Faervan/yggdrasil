//! This crate is used as networking library in yggrasil
use std::time::Duration;

use bevy_math::{Quat, Vec3};
use bevy_utils::HashMap;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use yserde_bytes::AsBytes;

/// functions and trait imlementations for use with the client side
pub mod client;
/// functions and trait imlementations for use with the server side
pub mod server;

#[cfg(test)]
mod tests;

#[derive(AsBytes, Debug)]
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

#[derive(AsBytes, Default)]
struct LobbyConnectionRequest(String);

#[derive(AsBytes)]
enum LobbyConnectionResponse {
    Accept {
        client_id: u16,
        lobby: Lobby
    },
    Deny
}

#[derive(AsBytes, Default, Debug)]
pub enum LobbyUpdate {
    #[default]
    Connect,
    Disconnect,
    ConnectionInterrupt,
    Reconnect,
    Message,
}

#[derive(AsBytes, Default, Debug)]
pub enum GameUpdate {
    #[default]
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

#[derive(AsBytes, Debug)]
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
