use bevy::prelude::*;
use ysync::{client::TcpUpdate, ClientStatus, GameUpdate, LobbyUpdate, TcpFromClient, UdpPackage};

use crate::{game::online::{events::{DespawnPlayer, MovePlayer, PlayerAttack, PlayerJump, ReceivedWorld, RotatePlayer, ShareWorld, SpawnPlayer}, OnlineState}, ui::{chat::{MessageSendEvent, PendingMessages}, lobby::JoinGameButton, MenuData, NORMAL_BUTTON}, AppState};

use super::{LobbySocket, LobbyState};

#[allow(private_interfaces)]
pub fn get_lobby_events(
    mut socket: ResMut<LobbySocket>,
    mut pending_msgs: ResMut<PendingMessages>,
    mut commands: Commands,
    menu_nodes: Res<MenuData>,
    app_state: Res<State<AppState>>,
    online_state: Res<State<OnlineState>>,
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
        AppState::Lobby(LobbyState::InLobby) => true,
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
                            });
                        }
                        if game.host_id == socket.socket.client_id {
                            socket.socket.game_id = Some(game.game_id);
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
                            next_state.set(AppState::Lobby(LobbyState::InLobby));
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
                                if *online_state.get() == OnlineState::Host {
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

pub fn send_msg_to_lobby(
    mut event_reader: EventReader<MessageSendEvent>,
    socket: Res<LobbySocket>,
) {
    for event in event_reader.read() {
        let _ = socket.socket.tcp_send.send(TcpFromClient::Message(event.0.clone()));
    }
}
