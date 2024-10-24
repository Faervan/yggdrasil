use std::fmt::Display;

use bevy_utils::HashMap;
use yserde_bytes::AsBytes;

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
pub enum TcpFromServer {
    LobbyUpdate(LobbyUpdate),
    GameUpdate(GameUpdate)
}

#[derive(AsBytes, Default, Debug)]
pub struct LobbyConnectionRequest(pub String);

#[derive(AsBytes)]
pub enum LobbyConnectionResponse {
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

pub trait CustomDisplay {
    fn to_string(&self) -> String;
}

impl CustomDisplay for HashMap<u16, Client> {
    fn to_string(&self) -> String {
        match self.len() == 0 {
             true => "No clients logged in".to_string(),
             false => self.values().fold("".to_string(), |acc, client| {
                format!("{acc}{}: {} (in_game: {}, status: {})",
                    client.client_id,
                    client.name,
                    client.in_game,
                    client.status)
            })
         }
    }
}

#[derive(AsBytes, Default, Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Idle(u16),
    #[default]
    Active,
}

impl Display for ClientStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientStatus::Active => write!(f, "Active"),
            ClientStatus::Idle(since) => write!(f, "Idle since {since}s")
        }
    }
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

impl CustomDisplay for HashMap<u16, Game> {
    fn to_string(&self) -> String {
        match self.len() == 0 {
             true => "No games hosted".to_string(),
             false => self.values().fold("".to_string(), |acc, game| {
                format!("{acc}{}: {}, (hosted_by: #{}, password: {}, connected clients: {:?})",
                    game.game_id,
                    game.game_name,
                    game.host_id,
                    match &game.password {
                        Some(pw) => pw.as_str(),
                        None => "None"
                    },
                    game.clients)
            })
         }
    }
}

#[derive(AsBytes, Default, Debug)]
pub struct Lobby {
    pub client_count: u16,
    pub game_count: u16,
    pub clients: HashMap<u16, Client>,
    pub games: HashMap<u16, Game>,
}
