use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use con_selection::{NameInput, PlayerName};
use tokio::sync::oneshot::{channel, Receiver};
use ysync::{client::{ConnectionSocket, LobbyConnectionError, TcpPackage, TcpUpdate}, ClientStatus, GameUpdateData, Lobby, LobbyUpdateData};

use crate::{game::OnlineGame, AppState, Settings};

use self::con_selection::{build_con_selection, lobby_con_interaction, ReturnButton};

use super::{chat::{ChatInput, MessageSendEvent, PendingMessages}, despawn_camera, despawn_menu, helper::Textfield, spawn_camera, MenuData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

mod con_selection;

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ConnectionState {
    #[default]
    None,
    Connected,
    Connecting,
}
#[derive(Resource)]
struct ConnectionBuilder(Receiver<Result<(ConnectionSocket, Lobby), LobbyConnectionError>>);
#[derive(Resource)]
struct Runtime(tokio::runtime::Runtime);
#[derive(Resource)]
pub struct LobbySocket {
    client_nodes: HashMap<u16, Entity>,
    game_nodes: HashMap<u16, Entity>,
    lobby: Lobby,
    socket: ConnectionSocket,
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
            .init_state::<ConnectionState>()
            .insert_resource(Runtime(rt))
            .add_systems(OnEnter(AppState::MultiplayerLobby(crate::LobbyState::ConSelection)), (
                build_con_selection,
                spawn_camera,
            ))
            .add_systems(OnExit(AppState::MultiplayerLobby(crate::LobbyState::ConSelection)), (
                despawn_menu,
                despawn_camera,
            ))
            .add_systems(OnEnter(AppState::MultiplayerLobby(crate::LobbyState::InLobby)), (
                build_lobby,
                spawn_camera,
                connect_to_lobby,
            ))
            .add_systems(OnExit(AppState::MultiplayerLobby(crate::LobbyState::InLobby)), (
                despawn_menu,
                despawn_camera,
                disconnet_from_lobby.run_if(in_state(ConnectionState::Connected)),
            ))
            .add_systems(Update, (
                lobby_con_interaction,
            ).run_if(in_state(AppState::MultiplayerLobby(crate::LobbyState::ConSelection))))
            .add_systems(Update, (
                lobby_interaction,
                game_section_interaction.run_if(in_state(ConnectionState::Connected)),
                con_finished_check.run_if(in_state(ConnectionState::Connecting)),
                get_lobby_events.run_if(in_state(ConnectionState::Connected)),
            ).run_if(in_state(AppState::MultiplayerLobby(crate::LobbyState::InLobby))));
    }
}

fn build_lobby(mut commands: Commands) {
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
    commands.insert_resource(MenuData { entities: vec![return_btn, client_list, games_node, games_list] });
}

fn lobby_interaction(
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

fn game_section_interaction(
    socket: ResMut<LobbySocket>,
    name_input: Query<&Text, With<NameInput>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut online_state: ResMut<NextState<OnlineGame>>,
    mut host_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<HostGameButton>)>,
    mut join_interaction_query: Query<(&Interaction, &mut BackgroundColor, &JoinGameButton), (Changed<Interaction>, Without<HostGameButton>)>,
) {
    for (interaction, mut color) in &mut host_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let _ = socket.socket.tcp_send.send(TcpPackage::GameCreation { name: name_input.single().sections[0].value.clone(), with_password: false });
                app_state.set(AppState::InGame);
                online_state.set(OnlineGame::Host);
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
                let _ = socket.socket.tcp_send.send(TcpPackage::GameEntry(game_id.0));
                app_state.set(AppState::InGame);
                online_state.set(OnlineGame::Client);
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

fn connect_to_lobby(
    name: Res<PlayerName>,
    rt: Res<Runtime>,
    settings: Res<Settings>,
    mut next_state: ResMut<NextState<ConnectionState>>,
    mut commands: Commands,
) {
    println!("trying to connect");
    let name = name.0.clone();
    let (sender, receiver) = channel();
    let lobby_addr = match settings.local_lobby {
        true => "127.0.0.1:9983",
        false => "91.108.102.51:9983",
    };
    rt.0.spawn(async move {
        let socket = ConnectionSocket::build(lobby_addr, "0.0.0.0:9983", name).await;
        let _ = sender.send(socket);
    });
    next_state.set(ConnectionState::Connecting);
    commands.insert_resource(ConnectionBuilder(receiver));
}

fn con_finished_check(
    mut receiver: ResMut<ConnectionBuilder>,
    mut next_state: ResMut<NextState<ConnectionState>>,
    mut commands: Commands,
    mut pending_msgs: ResMut<PendingMessages>,
    menu_nodes: Res<MenuData>,
) {
    if let Ok(result) = receiver.0.try_recv() {
        match result {
            Ok((socket, lobby)) => {
                println!("Connected!\n{socket:?}\n{lobby:#?}");
                next_state.set(ConnectionState::Connected);
                pending_msgs.0.push(format!("[INFO] Connected to lobby as #{}", socket.client_id));
                // Handle Clients
                let mut client_nodes = HashMap::new();
                commands.entity(menu_nodes.entities[1]).with_children(|p| {
                    for (_, client) in lobby.clients.iter() {
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
                    for (_, game) in lobby.games.iter() {
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
                commands.insert_resource(LobbySocket {client_nodes, game_nodes, lobby, socket});
            }
            Err(e) => {
                println!("error with connection: {e}");
                next_state.set(ConnectionState::None);
                pending_msgs.0.push(format!("[INFO] Failed to connect to lobby"));
            }
        }
        commands.remove_resource::<ConnectionBuilder>();
    }
}

fn get_lobby_events(
    mut socket: ResMut<LobbySocket>,
    mut pending_msgs: ResMut<PendingMessages>,
    mut commands: Commands,
    menu_nodes: Res<MenuData>,
) {
    for _ in 0..socket.socket.tcp_recv.len() {
        match socket.socket.tcp_recv.try_recv() {
            Ok(TcpUpdate::LobbyUpdate(update_type)) => {
                match update_type {
                    LobbyUpdateData::Connect(client) => {
                        pending_msgs.0.push(format!("[INFO] {} joined the lobby as #{}", client.name, client.client_id));
                        commands.entity(menu_nodes.entities[1]).with_children(|p| {
                            let id = p.spawn(TextBundle::from_section(
                                format!("{} (#{})", client.name, client.client_id).as_str(),
                                TextStyle {
                                    font_size: 33.,
                                    color: Color::srgb(0.9, 0.9, 0.9),
                                    ..default()
                                }
                            )).id();
                            socket.client_nodes.insert(client.client_id, id);
                        });
                        socket.lobby.clients.insert(client.client_id, client);
                    }
                    LobbyUpdateData::Disconnect(client_id) => {
                        pending_msgs.0.push(format!("[INFO] {} disconnected from the lobby", socket.lobby.clients.get(&client_id).unwrap().name));
                        socket.lobby.clients.remove(&client_id);
                        if let Some(entity) = socket.client_nodes.remove(&client_id) {
                            commands.entity(entity).despawn();
                        }
                    }
                    LobbyUpdateData::ConnectionInterrupt(client_id) => {
                        if let Some(client) = socket.lobby.clients.get_mut(&client_id) {
                            pending_msgs.0.push(format!("[INFO] connection to {} was interrupted", client.name));
                            client.status = ClientStatus::Idle(Duration::from_secs(0));
                            commands.entity(*socket.client_nodes.get(&client_id).unwrap()).add(|mut entity: EntityWorldMut| {
                                if let Some(mut text) = entity.get_mut::<Text>() {
                                    text.sections[0].style.color = Color::srgb(0.5, 0.5, 0.5);
                                }
                            });
                        }
                    }
                    LobbyUpdateData::Reconnect(client_id) => {
                        if let Some(client) = socket.lobby.clients.get_mut(&client_id) {
                            pending_msgs.0.push(format!("[INFO] {} reconnected to the lobby", client.name));
                            client.status = ClientStatus::Active;
                            commands.entity(*socket.client_nodes.get(&client_id).unwrap()).add(|mut entity: EntityWorldMut| {
                                if let Some(mut text) = entity.get_mut::<Text>() {
                                    text.sections[0].style.color = Color::srgb(0.9, 0.9, 0.9);
                                }
                            });
                        }
                    }
                    LobbyUpdateData::Message {sender: client_id, content, ..} => {
                        if client_id != socket.socket.client_id {
                            pending_msgs.0.push(format!("{}: {content}", socket.lobby.clients.get(&client_id).unwrap().name));
                        }
                    }
                }
            }
            Ok(TcpUpdate::GameUpdate(update_type)) => {
                match update_type {
                    GameUpdateData::Creation(game) => {
                        pending_msgs.0.push(format!(
                            "[INFO] {} hosted game {} (#{})",
                            socket.lobby.clients.get(&game.host_id).unwrap().name,
                            game.game_id,
                            game.game_name
                        ));
                        commands.entity(menu_nodes.entities[3]).with_children(|p| {
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
                            socket.game_nodes.insert(game.game_id, id);
                        });
                        socket.lobby.games.insert(game.game_id, game);
                    }
                    GameUpdateData::Deletion(game_id) => {
                        let game = socket.lobby.games.remove(&game_id).unwrap();
                        pending_msgs.0.push(format!(
                            "[INFO] game {} (#{}) was deleted",
                            game.game_name,
                            game_id,
                        ));
                        if let Some(entity) = socket.game_nodes.remove(&game_id) {
                            commands.entity(entity).despawn();
                        }
                    }
                    GameUpdateData::Entry { client_id, game_id } => {
                        if let Some(game) = socket.lobby.games.get_mut(&game_id) {
                            game.clients.push(client_id);
                            pending_msgs.0.push(format!(
                                "[INFO] client#{} joined game {} (#{})",
                                client_id,
                                game.game_name,
                                game_id,
                            ));
                        }
                    }
                    GameUpdateData::Exit(client_id) => {
                        if let Some(game) = socket.lobby.games.values_mut().find(|game| game.clients.contains(&client_id)) {
                            game.clients.retain(|c| *c != client_id);
                            pending_msgs.0.push(format!(
                                "[INFO] client#{} left game {} (#{})",
                                client_id,
                                game.game_name,
                                game.game_id,
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                pending_msgs.0.push(format!("[ERR] there was an unexpected error: {e}"));
            }
        }
    }
}

pub fn send_msg_to_lobby(
    mut event_reader: EventReader<MessageSendEvent>,
    socket: Res<LobbySocket>,
) {
    for event in event_reader.read() {
        let _ = socket.socket.tcp_send.send(TcpPackage::Message(event.0.clone()));
    }
}

fn disconnet_from_lobby(
    lobby_socket: Res<LobbySocket>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ConnectionState>>,
) {
    let _ = lobby_socket.socket.tcp_send.send(TcpPackage::Disconnect);
    commands.remove_resource::<LobbySocket>();
    next_state.set(ConnectionState::None);
}
