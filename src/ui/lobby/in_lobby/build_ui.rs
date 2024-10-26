use bevy::{prelude::*, utils::HashMap};

use crate::ui::{components::{NameInput, ReturnButton}, helper::Textfield, lobby::{HostGameButton, JoinGameButton, LobbySocket}, MenuData, NORMAL_BUTTON};

pub fn build_lobby(
    mut commands: Commands,
    mut menudata: ResMut<MenuData>,
) {
    let return_btn = commands.spawn((
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
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Return",
            TextStyle {
                font_size: 33.0,
                color: Color::srgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    }).id();
    let client_list = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::End,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Clients",
            TextStyle {
                font_size: 33.,
                color: Color::srgb(0.9, 0.9, 0.9),
                ..default()
            }
        ));
    }).id();
    let mut games_list: Entity = Entity::from_raw(0);
    let games_node = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // Active games list
        games_list = parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(500.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Games",
                TextStyle {
                    font_size: 33.,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                }
            ));
        }).id();
        // Inout field to set game name
        parent.spawn_empty().as_textfield("Game name", NameInput {}, Val::Px(250.), Val::Px(65.), Some(UiRect::DEFAULT.with_top(Val::Px(15.))), 33.);
        // Button to host game
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(250.),
                    height: Val::Px(65.),
                    margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(5.), Val::ZERO),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            HostGameButton {}
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Host game",
                TextStyle {
                    font_size: 33.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });
    }).id();
    menudata.entities = vec![return_btn, client_list, games_node, games_list];
}

pub fn build_lobby_details(
    mut remote: ResMut<LobbySocket>,
    mut commands: Commands,
    menu_nodes: Res<MenuData>,
) {
    // Handle Clients
    let mut client_nodes = HashMap::new();
    commands.entity(menu_nodes.entities[1]).with_children(|p| {
        for (_, client) in remote.lobby.clients.iter() {
            let id = p.spawn(TextBundle::from_section(
                format!("{} (#{})", client.name, client.client_id).as_str(),
                TextStyle {
                    font_size: 33.,
                    color: match client.status {
                        ysync::ClientStatus::Active => Color::srgb(0.9, 0.9, 0.9),
                        ysync::ClientStatus::Idle(_) => Color::srgb(0.5, 0.5, 0.5),
                    },
                    ..default()
                }
            )).id();
            client_nodes.insert(client.client_id, id);
        }
    });
    // Handle Games
    let mut game_nodes = HashMap::new();
    commands.entity(menu_nodes.entities[3]).with_children(|p| {
        for (_, game) in remote.lobby.games.iter() {
            let id = p.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(55.),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            }).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    format!("{} (#{})", game.game_name, game.game_id).as_str(),
                    TextStyle {
                        font_size: 33.,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    }
                ));
                p.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.),
                            height: Val::Percent(100.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    JoinGameButton(game.game_id),
                )).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Join",
                        TextStyle {
                            font_size: 33.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
            }).id();
            game_nodes.insert(game.game_id, id);
        }
    });
    remote.game_nodes = game_nodes;
    remote.client_nodes = client_nodes;
}
