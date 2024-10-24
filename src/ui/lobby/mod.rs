use bevy::{prelude::*, utils::HashMap};
use con_selection::NameInput;
use tokio::sync::oneshot::{channel, Receiver};
use ysync::{client::{ConnectionSocket, LobbyConnectionError, TcpUpdate}, ClientStatus, GameUpdate, Lobby, LobbyUpdate, TcpFromClient, UdpPackage};

use crate::{game::{OnlineGame, PlayerId, PlayerName}, AppState, DespawnPlayer, LobbyState, MovePlayer, PlayerAttack, PlayerJump, ReceivedWorld, RotatePlayer, Settings, ShareWorld, SpawnPlayer};

use self::con_selection::{build_con_selection, lobby_con_interaction, ReturnButton};

use super::{chat::{MessageSendEvent, PendingMessages}, despawn_camera, despawn_menu, helper::Textfield, spawn_camera, MenuData, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

mod con_selection;
mod awaiting_join_permission;

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
            .init_state::<ConnectionState>()
            .insert_resource(Runtime(rt))
            .add_systems(OnEnter(AppState::MultiplayerLobby(LobbyState::ConSelection)), (
                build_con_selection,
                spawn_camera,
            ))
            .add_systems(OnExit(AppState::MultiplayerLobby(LobbyState::ConSelection)), (
                despawn_menu,
                despawn_camera,
            ))
            .add_systems(OnEnter(AppState::MultiplayerLobby(LobbyState::InLobby)), (
                build_lobby,
                build_lobby_details.run_if(resource_exists::<LobbySocket>).after(build_lobby),
                spawn_camera,
            ))
            .add_systems(OnTransition {
                exited: AppState::MultiplayerLobby(LobbyState::ConSelection),
                entered: AppState::MultiplayerLobby(LobbyState::InLobby),
            }, connect_to_lobby)
            .add_systems(OnExit(AppState::MultiplayerLobby(LobbyState::InLobby)), (
                despawn_menu,
                despawn_camera,
            ))
            .add_systems(OnTransition {
                exited: AppState::MultiplayerLobby(LobbyState::InLobby),
                entered: AppState::MainMenu,
            }, disconnect_from_lobby.run_if(in_state(ConnectionState::Connected)))
            .add_systems(Update, (
                get_lobby_events.run_if(resource_exists::<LobbySocket>)
            ).run_if(in_state(ConnectionState::Connected)))
            .add_systems(Update, (
                lobby_con_interaction,
            ).run_if(in_state(AppState::MultiplayerLobby(LobbyState::ConSelection))))
            .add_systems(OnEnter(ConnectionState::Connected), build_lobby_details)
            .add_systems(Update, (
                lobby_interaction,
                game_section_interaction.run_if(in_state(ConnectionState::Connected)),
                con_finished_check.run_if(in_state(ConnectionState::Connecting)),
            ).run_if(in_state(AppState::MultiplayerLobby(LobbyState::InLobby))));
    }
}

fn build_lobby(
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
    mut socket: ResMut<LobbySocket>,
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
                let _ = socket.socket.tcp_send.send(TcpFromClient::GameCreation { name: name_input.single().sections[0].value.clone(), password: None });
                app_state.set(AppState::InGame);
                online_state.set(OnlineGame::Host);
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
                app_state.set(AppState::MultiplayerLobby(LobbyState::AwaitingJoinPermission));
                online_state.set(OnlineGame::Client);
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
        true => "127.0.0.1:9983".to_string(),
        false => settings.lobby_url.clone(),
    };
    rt.0.spawn(async move {
        let socket = ConnectionSocket::build(lobby_addr, "0.0.0.0:0".to_string(), name).await;
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
    mut player_id: ResMut<PlayerId>,
) {
    if let Ok(result) = receiver.0.try_recv() {
        match result {
            Ok((socket, lobby)) => {
                println!("Connected!\n{socket:?}\n{lobby:#?}");
                player_id.0 = socket.client_id;
                next_state.set(ConnectionState::Connected);
                pending_msgs.0.push(format!("[INFO] Connected to lobby as #{}", socket.client_id));
                commands.insert_resource(LobbySocket {client_nodes: HashMap::new(), game_nodes: HashMap::new(), lobby, socket});
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
    app_state: Res<State<AppState>>,
    online_state: Res<State<OnlineGame>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut share_world_event: EventWriter<ShareWorld>,
    mut received_world_event: EventWriter<ReceivedWorld>,
    mut player_spawn_event: EventWriter<SpawnPlayer>,
    mut player_despawn_event: EventWriter<DespawnPlayer>,
    mut player_attack_event: EventWriter<PlayerAttack>,
    mut player_move_event: EventWriter<MovePlayer>,
    mut player_rotate_event: EventWriter<RotatePlayer>,
    mut player_jump_event: EventWriter<PlayerJump>,
) {
    let in_lobby = match app_state.get() {
        AppState::MultiplayerLobby(LobbyState::InLobby) => true,
        _ => false,
    };
    for _ in 0..socket.socket.tcp_recv.len() {
        match socket.socket.tcp_recv.try_recv() {
            Ok(TcpUpdate::LobbyUpdate(update_type)) => {
                match update_type {
                    LobbyUpdate::Connection(client) => {
                        pending_msgs.0.push(format!("[INFO] {} joined the lobby as #{}", client.name, client.client_id));
                        if in_lobby {
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
                        }
                        socket.lobby.clients.insert(client.client_id, client);
                    }
                    LobbyUpdate::Disconnection(client_id) => {
                        pending_msgs.0.push(format!("[INFO] {} disconnected from the lobby", socket.lobby.clients.get(&client_id).unwrap().name));
                        socket.lobby.clients.remove(&client_id);
                        if let Some(entity) = socket.client_nodes.remove(&client_id) {
                            if in_lobby {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                    LobbyUpdate::ConnectionInterrupt(client_id) => {
                        if let Some(client) = socket.lobby.clients.get_mut(&client_id) {
                            pending_msgs.0.push(format!("[INFO] connection to {} was interrupted", client.name));
                            client.status = ClientStatus::Idle(0);
                            if in_lobby {
                                commands.entity(*socket.client_nodes.get(&client_id).unwrap()).add(|mut entity: EntityWorldMut| {
                                    if let Some(mut text) = entity.get_mut::<Text>() {
                                        text.sections[0].style.color = Color::srgb(0.5, 0.5, 0.5);
                                    }
                                });
                            }
                        }
                    }
                    LobbyUpdate::Reconnect(client_id) => {
                        if let Some(client) = socket.lobby.clients.get_mut(&client_id) {
                            pending_msgs.0.push(format!("[INFO] {} reconnected to the lobby", client.name));
                            client.status = ClientStatus::Active;
                            if in_lobby {
                                commands.entity(*socket.client_nodes.get(&client_id).unwrap()).add(|mut entity: EntityWorldMut| {
                                    if let Some(mut text) = entity.get_mut::<Text>() {
                                        text.sections[0].style.color = Color::srgb(0.9, 0.9, 0.9);
                                    }
                                });
                            }
                        }
                    }
                    LobbyUpdate::Message {sender: client_id, content, ..} => {
                        if client_id != socket.socket.client_id {
                            pending_msgs.0.push(format!("{}: {content}", socket.lobby.clients.get(&client_id).unwrap().name));
                        }
                    }
                    LobbyUpdate::Default => {
                        println!("got a GameUpdate::Default ... this should not have happened!")
                    }
                }
            }
            Ok(TcpUpdate::GameUpdate(update_type)) => {
                match update_type {
                    GameUpdate::Creation(game) => {
                        pending_msgs.0.push(format!(
                            "[INFO] {} hosted game {} (#{})",
                            socket.lobby.clients.get(&game.host_id).unwrap().name,
                            game.game_name,
                            game.game_id
                        ));
                        if in_lobby {
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
                                if game.host_id == socket.socket.client_id {
                                    socket.socket.game_id = Some(game.game_id);
                                }
                            });
                        }
                        socket.lobby.games.insert(game.game_id, game);
                    }
                    GameUpdate::Deletion(game_id) => {
                        let game = socket.lobby.games.remove(&game_id).unwrap();
                        pending_msgs.0.push(format!(
                            "[INFO] game {} (#{}) was deleted",
                            game.game_name,
                            game_id,
                        ));
                        if in_lobby {
                            if let Some(entity) = socket.game_nodes.remove(&game_id) {
                                commands.entity(entity).despawn_recursive();
                            }
                        } else if game.clients.contains(&socket.socket.client_id) {
                            pending_msgs.0.push("[INFO] Host closed your game!".to_string());
                            next_state.set(AppState::MultiplayerLobby(LobbyState::InLobby));
                        }
                    }
                    GameUpdate::Entry { client_id, game_id } => {
                        if let Some(game) = socket.lobby.games.get_mut(&game_id) {
                            game.clients.push(client_id);
                            pending_msgs.0.push(format!(
                                "[INFO] client#{} joined game {} (#{})",
                                client_id,
                                game.game_name,
                                game_id,
                            ));
                            if Some(game.game_id) == socket.socket.game_id {
                                if client_id != socket.socket.client_id {
                                    player_spawn_event.send(SpawnPlayer {
                                        name: socket.lobby.clients.get(&client_id).unwrap().name.clone(),
                                        id: client_id,
                                        position: Transform::from_xyz(0., 10., 0.).with_scale(Vec3::new(0.4, 0.4, 0.4))
                                    });
                                }
                                if *online_state.get() == OnlineGame::Host {
                                    share_world_event.send(ShareWorld);
                                }
                            }
                        }
                    }
                    GameUpdate::Exit(client_id) => {
                        if let Some(game) = socket.lobby.games.values_mut().find(|game| game.clients.contains(&client_id)) {
                            game.clients.retain(|c| *c != client_id);
                            pending_msgs.0.push(format!(
                                "[INFO] client#{} left game {} (#{})",
                                client_id,
                                game.game_name,
                                game.game_id,
                            ));
                            if Some(game.game_id) == socket.socket.game_id {
                                player_despawn_event.send(DespawnPlayer(client_id));
                            }
                        }
                    }
                    GameUpdate::World(scene) => {
                        println!("received TcpUpdate::GameWorld, sending ReceivedWorld event...");
                        received_world_event.send(ReceivedWorld(scene));
                        let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                        let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                        let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                        let _ = socket.socket.udp_send.send(UdpPackage::Heartbeat);
                    }
                    GameUpdate::Default => {
                        println!("got a GameUpdate::Default ... this should not have happened!")
                    }
                }
            }
            Err(e) => {
                pending_msgs.0.push(format!("[ERR] there was an unexpected error: {e}"));
            }
        }
    }
    for _ in 0..socket.socket.udp_recv.len() {
        match socket.socket.udp_recv.try_recv() {
            Ok(udp_from_server) => match udp_from_server.data {
                UdpPackage::Attack(ypos) => {
                    player_attack_event.send(PlayerAttack {
                        player_id: udp_from_server.sender_id,
                        position: Transform::from(ypos)
                    });
                }
                UdpPackage::Move(pos) => {
                    player_move_event.send(MovePlayer { id: udp_from_server.sender_id, position: Vec3::from(pos) });
                }
                UdpPackage::Rotate(rotation) => {
                    player_rotate_event.send(RotatePlayer { id: udp_from_server.sender_id, rotation: Quat::from(rotation) });
                }
                UdpPackage::Jump => {
                    player_jump_event.send(PlayerJump(udp_from_server.sender_id));
                }
                _ => {
                    pending_msgs.0.push(format!("[ERR] there was an unexpected udp package"));
                }
            }
            Err(e) => {
                pending_msgs.0.push(format!("[ERR] there was an unexpected error: {e}"));
            }
        }
    }
}

fn build_lobby_details(
    mut socket: ResMut<LobbySocket>,
    mut commands: Commands,
    menu_nodes: Res<MenuData>,
) {
    // Handle Clients
    let mut client_nodes = HashMap::new();
    commands.entity(menu_nodes.entities[1]).with_children(|p| {
        for (_, client) in socket.lobby.clients.iter() {
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
        for (_, game) in socket.lobby.games.iter() {
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
    socket.game_nodes = game_nodes;
    socket.client_nodes = client_nodes;
}

pub fn send_msg_to_lobby(
    mut event_reader: EventReader<MessageSendEvent>,
    socket: Res<LobbySocket>,
) {
    for event in event_reader.read() {
        let _ = socket.socket.tcp_send.send(TcpFromClient::Message(event.0.clone()));
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
