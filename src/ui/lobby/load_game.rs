use bevy::prelude::*;
use ysync::TcpFromClient;

use crate::{game::online::OnlineState, ui::{components::ReturnButton, despawn_camera, despawn_menu, spawn_camera, MenuData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON}, AppState, LobbyState};

use super::LobbySocket;

pub struct GameLoadPlugin;

impl Plugin for GameLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Lobby(LobbyState::LoadGame)), (
                build_load_screen,
                spawn_camera,
            ))
            .add_systems(OnExit(AppState::Lobby(LobbyState::LoadGame)), (
                despawn_menu,
                despawn_camera,
            ))
            .add_systems(Update, (
                cancel_interaction,
            ).run_if(in_state(AppState::Lobby(LobbyState::LoadGame))));
    }
}

fn build_load_screen(mut commands: Commands) {
    let entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Cancel button
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(250.),
                            height: Val::Px(65.),
                            margin: UiRect::ZERO,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    ReturnButton {}
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Cancel",
                        TextStyle {
                            font_size: 33.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        }).id();
    commands.insert_resource(MenuData { entities: vec![entity] });
}

fn cancel_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut return_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ReturnButton>)>,
    mut online_state: ResMut<NextState<OnlineState>>,
    mut remote: ResMut<LobbySocket>,
) {
    for (interaction, mut color) in &mut return_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::Lobby(LobbyState::InLobby));
                let _ = remote.socket.tcp_send.send(TcpFromClient::GameExit);
                remote.socket.game_id = None;
                online_state.set(OnlineState::Client);
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
