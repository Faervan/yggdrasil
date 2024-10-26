use bevy::prelude::*;
use ysync::{TcpFromClient, UdpPackage};

use crate::{game::online::OnlineState, ui::{components::{NameInput, ReturnButton}, lobby::{HostGameButton, JoinGameButton, LobbySocket, LobbyState}, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON}, AppState};

pub fn lobby_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut return_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ReturnButton>)>,
) {
    for (interaction, mut color) in &mut return_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::MainMenu);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn game_section_interaction(
    mut socket: ResMut<LobbySocket>,
    name_input: Query<&Text, With<NameInput>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut online_state: ResMut<NextState<OnlineState>>,
    mut host_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<HostGameButton>)>,
    mut join_interaction_query: Query<(&Interaction, &mut BackgroundColor, &JoinGameButton), (Changed<Interaction>, Without<HostGameButton>)>,
) {
    for (interaction, mut color) in &mut host_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let _ = socket.socket.tcp_send.send(TcpFromClient::GameCreation { name: name_input.single().sections[0].value.clone(), password: None });
                app_state.set(AppState::InGame);
                online_state.set(OnlineState::Host);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
    for (interaction, mut color, game_id) in &mut join_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let _ = socket.socket.tcp_send.send(TcpFromClient::GameEntry { password: None, game_id: game_id.0 });
                socket.socket.game_id = Some(game_id.0);
                app_state.set(AppState::Lobby(LobbyState::LoadGame));
                online_state.set(OnlineState::Client);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
