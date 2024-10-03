use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use con_selection::PlayerName;
use tokio::sync::oneshot::{channel, Receiver};
use ysync::{client::{ConnectionSocket, LobbyConnectionError, TcpPackage}, ClientStatus, Lobby, LobbyUpdateData};

use crate::AppState;

use self::con_selection::{build_con_selection, lobby_con_interaction, ReturnButton};

use super::{chat::PendingMessages, despawn_camera, despawn_menu, spawn_camera, MenuData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

mod con_selection;

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum ConnectionState {
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
struct LobbySocket {
    client_nodes: HashMap<u16, Entity>,
    lobby: Lobby,
    socket: ConnectionSocket,
}

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
    commands.insert_resource(MenuData { entities: vec![return_btn, client_list] });
}

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

fn connect_to_lobby(
    name: Res<PlayerName>,
    rt: Res<Runtime>,
    mut next_state: ResMut<NextState<ConnectionState>>,
    mut commands: Commands,
) {
    println!("trying to connect");
    let name = name.0.clone();
    let (sender, receiver) = channel();
    rt.0.spawn(async move {
        let socket = ConnectionSocket::build("91.108.102.51:9983", "0.0.0.0:9983", name).await;
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
                commands.insert_resource(LobbySocket {client_nodes, lobby, socket});
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
            Ok(LobbyUpdateData::Connect(client)) => {
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
            Ok(LobbyUpdateData::Disconnect(client_id)) => {
                pending_msgs.0.push(format!("[INFO] {} disconnected from the lobby", socket.lobby.clients.get(&client_id).unwrap().name));
                socket.lobby.clients.remove(&client_id);
                if let Some(entity) = socket.client_nodes.remove(&client_id) {
                    commands.entity(entity).despawn();
                }
            }
            Ok(LobbyUpdateData::ConnectionInterrupt(client_id)) => {
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
            Ok(LobbyUpdateData::Reconnect(client_id)) => {
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
            Ok(LobbyUpdateData::Message {sender: client_id, content, ..}) => {
                pending_msgs.0.push(format!("{}: {content}", socket.lobby.clients.get(&client_id).unwrap().name));
            }
            Err(e) => {
                pending_msgs.0.push(format!("[ERR] there was an unexpected error: {e}"));
            }
        }
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
