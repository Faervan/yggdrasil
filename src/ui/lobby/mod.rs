use bevy::{prelude::*, utils::HashMap};
use con_menu::LobbyConMenuPlugin;
use in_lobby::InLobbyPlugin;
use load_game::GameLoadPlugin;
use tokio::sync::oneshot::Receiver;
use ysync::{client::{ConnectionSocket, LobbyConnectionError}, Lobby};

mod con_menu;
pub mod in_lobby;
mod load_game;

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ConnectionState {
    #[default]
    None,
    Connected,
    Connecting,
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum LobbyState {
    #[default]
    ConMenu,
    InLobby,
    LoadGame,
}

#[derive(Resource)]
struct ConnectionBuilder(Receiver<Result<(ConnectionSocket, Lobby), LobbyConnectionError>>);
#[derive(Resource)]
struct Runtime(tokio::runtime::Runtime);
#[derive(Resource)]
pub struct LobbySocket {
    client_nodes: HashMap<u16, Entity>,
    game_nodes: HashMap<u16, Entity>,
    pub lobby: Lobby,
    pub socket: ConnectionSocket,
}
#[derive(Component)]
struct HostGameButton;
#[derive(Component)]
struct JoinGameButton(u16);

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        app
            .add_plugins((
                GameLoadPlugin,
                LobbyConMenuPlugin,
                InLobbyPlugin,
            ))
            .init_state::<ConnectionState>()
            .insert_resource(Runtime(rt));
    }
}
