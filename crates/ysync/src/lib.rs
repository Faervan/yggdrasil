//! This crate is used as networking library in yggrasil
use std::fmt::Display;

//use bevy_math::{Quat, Vec3};
use bevy_utils::HashMap;
use yserde_bytes::AsBytes;

/// functions and trait imlementations for use with the client side
pub mod client;
/// functions and trait imlementations for use with the server side
pub mod server;

#[cfg(test)]
mod tests;

#[derive(AsBytes, Debug)]
pub enum TcpFromClient {
    LobbyDisconnect,
    GameCreation {
        password: Option<String>,
        name: String
    },
    GameDeletion,
    GameEntry {
        password: Option<String>,
        game_id: u16
    },
    GameExit,
    GameWorld(#[u16]String),
    Message(String),
}

#[derive(AsBytes, Debug)]
enum TcpFromServer {
    LobbyUpdate(LobbyUpdate),
    GameUpdate(GameUpdate)
}

#[derive(AsBytes, Default)]
struct LobbyConnectionRequest(String);

#[derive(AsBytes)]
enum LobbyConnectionResponse {
    Accept {
        client_id: u16,
        lobby: Lobby
    },
    Deny(LobbyConnectionDenyReason)
}

#[derive(AsBytes, Default, Debug)]
pub enum LobbyConnectionDenyReason {
    #[default]
    AlreadyConnected
}

impl Display for LobbyConnectionDenyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyConnected => write!(f, "There already is an active connection with this IP")
        }
    }
}

#[derive(AsBytes, Default, Debug, PartialEq, Eq, Clone)]
pub enum LobbyUpdate {
    #[default]
    Default,
    Connection(Client),
    Disconnection(u16),
    ConnectionInterrupt(u16),
    Reconnect(u16),
    Message {
        sender: u16,
        content: String
    },
}

#[derive(AsBytes, Default, Debug, PartialEq, Eq, Clone)]
pub enum GameUpdate {
    #[default]
    Default,
    Creation(Game),
    Deletion(u16),
    Entry {
        client_id: u16,
        game_id: u16
    },
    Exit(u16),
    World(#[u16]String),
}

#[derive(AsBytes, Default, Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub client_id: u16,
    pub in_game: bool,
    pub status: ClientStatus,
    pub name: String,
}

#[derive(AsBytes, Default, Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Idle(u16),
    #[default]
    Active,
}

impl Client {
    pub fn new(name: String) -> Client {
        Client { client_id: 0, in_game: false, status: ClientStatus::Active, name }
    }
}

#[derive(AsBytes, Default, Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub game_id: u16,
    pub host_id: u16,
    pub password: Option<String>,
    pub game_name: String,
    pub clients: Vec<u16>,
}

#[derive(AsBytes, Default, Debug)]
pub struct Lobby {
    pub client_count: u16,
    pub game_count: u16,
    pub clients: HashMap<u16, Client>,
    pub games: HashMap<u16, Game>,
}
