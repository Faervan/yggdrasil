use bevy::{prelude::*, utils::HashMap};
use tokio::sync::oneshot::channel;
use ysync::client::ConnectionSocket;

use crate::{game::base::resources::{PlayerId, PlayerName}, ui::{chat::PendingMessages, lobby::{ConnectionBuilder, LobbySocket}}, Settings};

use super::super::{ConnectionState, Runtime};

pub fn connect_to_lobby(
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

pub fn con_finished_check(
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
