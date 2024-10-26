use bevy::prelude::*;
use connecting::{con_finished_check, connect_to_lobby};

use crate::{game::base::resources::PlayerName, ui::{components::{JoinButton, NameInput, ReturnButton}, despawn_camera, despawn_menu, helper::{TextFieldContent, Textfield}, spawn_camera, MenuData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON}, AppState};

use super::{ConnectionState, LobbyState};

mod connecting;

pub struct LobbyConMenuPlugin;

impl Plugin for LobbyConMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Lobby(LobbyState::ConMenu)), (
                build_con_menu,
                spawn_camera
            ))
            .add_systems(OnExit(AppState::Lobby(LobbyState::ConMenu)), (
                despawn_menu,
                despawn_camera,
            ))
            .add_systems(OnTransition {
                exited: AppState::Lobby(LobbyState::ConMenu),
                entered: AppState::Lobby(LobbyState::InLobby),
            }, connect_to_lobby)
            .add_systems(Update, (
                menu_interaction.run_if(in_state(AppState::Lobby(LobbyState::ConMenu))),
                con_finished_check.run_if(in_state(ConnectionState::Connecting)),
            ));
    }
}

fn build_con_menu(mut commands: Commands) {
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
            // name text field
            parent.spawn_empty().as_textfield("Your name", NameInput {}, Val::Px(250.), Val::Px(65.), None, 33.);
            // join button
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(250.),
                            height: Val::Px(65.),
                            margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(5.), Val::ZERO),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    JoinButton {}
                ))
                .with_children(|parent| {
                    let style = TextStyle {
                        font_size: 33.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    };
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new("Join ", style.clone()),
                        TextSection::new("lobby", style.clone()),
                    ]));
                });
            // Cancel button
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(250.),
                            height: Val::Px(65.),
                            margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(5.), Val::ZERO),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
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

fn menu_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut join_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<JoinButton>)>,
    mut return_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ReturnButton>, Without<JoinButton>)>,
    name_field: Query<&TextFieldContent, With<NameInput>>,
    mut player_name: ResMut<PlayerName>,
) {
    for (interaction, mut color) in &mut join_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let name: String = name_field.get_single().unwrap().0.to_string();
                if !name.is_empty() {
                    println!("name is {name}");
                    player_name.0 = name;
                    next_state.set(AppState::Lobby(LobbyState::InLobby));
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
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
