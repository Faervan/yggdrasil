use bevy::prelude::*;
use build_ui::{build_lobby, build_lobby_details};
use interaction::{game_section_interaction, lobby_interaction};
use read_packets::get_lobby_events;
use ysync::TcpFromClient;

use crate::{ui::{despawn_camera, despawn_menu, spawn_camera}, AppState};

use super::{ConnectionState, LobbySocket, LobbyState};

mod build_ui;
mod interaction;
pub mod read_packets;

pub struct InLobbyPlugin;

impl Plugin for InLobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Lobby(LobbyState::InLobby)), (
                build_lobby,
                build_lobby_details.run_if(resource_exists::<LobbySocket>).after(build_lobby),
                spawn_camera,
            ))
            .add_systems(OnExit(AppState::Lobby(LobbyState::InLobby)), (
                despawn_menu,
                despawn_camera,
            ))
            .add_systems(OnTransition {
                exited: AppState::Lobby(LobbyState::InLobby),
                entered: AppState::MainMenu,
            }, disconnect_from_lobby.run_if(in_state(ConnectionState::Connected)))
            .add_systems(Update,
                get_lobby_events
            .run_if(in_state(ConnectionState::Connected)))
            .add_systems(OnEnter(ConnectionState::Connected), build_lobby_details)
            .add_systems(Update, (
                lobby_interaction,
                game_section_interaction.run_if(in_state(ConnectionState::Connected)),
            ).run_if(in_state(AppState::Lobby(LobbyState::InLobby))));
    }
}

fn disconnect_from_lobby(
    lobby_socket: Res<LobbySocket>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ConnectionState>>,
) {
    let _ = lobby_socket.socket.tcp_send.send(TcpFromClient::LobbyDisconnect);
    commands.remove_resource::<LobbySocket>();
    next_state.set(ConnectionState::None);
}
