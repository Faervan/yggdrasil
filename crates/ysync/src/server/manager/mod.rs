use std::net::IpAddr;

use bevy_utils::HashMap;
use client_manager::ClientManager;
use game_manager::GameManager;
use tokio::sync::{broadcast::Sender, mpsc::UnboundedReceiver};

use crate::{Client, Game};

use super::EventBroadcast;

mod client_manager;
mod game_manager;

pub enum ManagerNotify {
    Connected {
        addr: IpAddr,
        client: Client,
    },
    Disconnected(IpAddr),
    ConnectionInterrupt(IpAddr),
    Message {
        client_id: u16,
        content: String
    },
    GameCreation(Game),
    GameDeletion(/*host_id:*/u16),
    GameEntry {
        password: Option<String>,
        client_id: u16,
        game_id: u16,
    },
    GameExit(/*client_id:*/u16),
    GameWorld {
        client_id: u16,
        scene: String,
    },
}

pub async fn client_game_manager(
    client_event: Sender<EventBroadcast>,
    client_list: Sender<HashMap<u16, Client>>,
    game_list: Sender<HashMap<u16, Game>>,
    mut receiver: UnboundedReceiver<ManagerNotify>
) -> tokio::io::Result<()> {
    let mut client_manager = ClientManager::new();
    let mut game_manager = GameManager::new();
    loop {
        match receiver.recv().await {
            Some(ManagerNotify::Connected { addr, mut client }) => {
                println!("{} connected! addr: {addr}", client.name);
                if let Some(reconnect) = client_manager.add_client(&mut client, addr) {
                    match reconnect {
                        true => {
                            let _ = client_event.send(EventBroadcast::Reconnected {addr, client});
                        }
                        false => {
                            let _ = client_event.send(EventBroadcast::Connected {addr, client});
                        }
                    }
                } else {
                    println!("client {addr} is already connected!");
                    let _ = client_event.send(EventBroadcast::Multiconnect(addr));
                }
            }
            Some(ManagerNotify::Disconnected(addr)) => {
                println!("client disconnected! addr: {addr}");
                let client_id = client_manager.remove_client(addr);
                game_manager.remove_game(client_id);
                let _ = client_event.send(EventBroadcast::Disconnected(client_id));
            }
            Some(ManagerNotify::ConnectionInterrupt(addr)) => {
                println!("Connection with {addr} has been interrupted!");
                let client_id = client_manager.inactivate_client(addr);
                let _ = client_event.send(EventBroadcast::ConnectionInterrupt(client_id));
            }
            Some(ManagerNotify::Message { client_id, content }) => {
                println!("{} (#{client_id}): {content}", client_manager.get_client(client_id).name);
                let _ = client_event.send(EventBroadcast::Message { client_id, content });
            }
            Some(ManagerNotify::GameCreation(mut game)) => {
                println!("{} (#{}) wants to create a game {}", game.host_id, client_manager.get_client(game.host_id).name, game.game_name);
                if game_manager.add_game(&mut game) {
                    let _ = client_event.send(EventBroadcast::GameCreation(game));
                }
            }
            Some(ManagerNotify::GameDeletion(host_id)) => {
                println!("{} (#{host_id}) wants to delete his game", client_manager.get_client(host_id).name);
                let game_id = game_manager.remove_game(host_id);
                let _ = client_event.send(EventBroadcast::GameDeletion(game_id.unwrap()));
            }
            Some(ManagerNotify::GameEntry { client_id, game_id, password }) => {
                println!("{} (#{client_id}) wants to join the game #{game_id} with password: {password:?}", client_manager.get_client(client_id).name);
                game_manager.add_client_to_game(client_id, game_id);
                let _ = client_event.send(EventBroadcast::GameEntry { client_id, game_id });
            }
            Some(ManagerNotify::GameExit(client_id)) => {
                println!("{} (#{client_id}) wants to leave his game", client_manager.get_client(client_id).name);
                game_manager.remove_client_from_game(client_id);
                let _ = client_event.send(EventBroadcast::GameExit(client_id));
            }
            Some(ManagerNotify::GameWorld { client_id, scene }) => {
                println!("{} (#{client_id}) shares his game world", client_manager.get_client(client_id).name);
                let _ = client_event.send(EventBroadcast::GameWorld { client_id, scene });
            }
            _ => println!("shit"),
        }
        let _ = client_list.send(client_manager.get_clients());
        let _ = game_list.send(game_manager.get_games());
    }
}
