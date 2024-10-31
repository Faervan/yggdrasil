use std::{net::IpAddr, time::Duration};

use bevy_utils::HashMap;
use client_manager::ClientManager;
use game_manager::GameManager;
use tokio::{sync::{broadcast::Sender, mpsc::{UnboundedReceiver, UnboundedSender}, oneshot}, time::{sleep_until, Instant}};

use crate::{Client, CustomDisplay, Game};

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
    GameCreation {
        game: Game,
        host_addr: IpAddr
    },
    GameDeletion(/*host_id:*/u16),
    GameEntry {
        password: Option<String>,
        client_id: u16,
        client_addr: IpAddr,
        game_id: u16,
    },
    GameExit(/*client_id:*/u16),
    GameWorld {
        client_id: u16,
        scene: String,
    },
    Command {
        response: oneshot::Sender<String>,
        value: String
    }
}

pub async fn client_game_manager(
    client_event: Sender<EventBroadcast>,
    client_list: Sender<HashMap<u16, Client>>,
    game_list: Sender<HashMap<u16, Game>>,
    mut receiver: UnboundedReceiver<ManagerNotify>,
    con_event_sender: UnboundedSender<ConnectionEvent>,
) -> tokio::io::Result<()> {
    let mut client_manager = ClientManager::new();
    let mut game_manager = GameManager::new();
    loop {
        let manager_notify = receiver.recv().await;
        match manager_notify.expect("ManagerNotify channel has been closed ... fuck") {
            ManagerNotify::Connected { addr, mut client } => {
                println!("{} connected! addr: {addr}", client.name);
                if let Some(reconnect) = client_manager.add_client(&mut client, addr) {
                    match reconnect {
                        true => {
                            let _ = client_event.send(EventBroadcast::Reconnected {addr, client});
                            let _ = con_event_sender.send(ConnectionEvent::Reconnect(addr));
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
            ManagerNotify::Disconnected(addr) => {
                println!("client disconnected! addr: {addr}");
                let client_id = client_manager.remove_client(addr);
                game_manager.remove_game(client_id);
                let _ = client_event.send(EventBroadcast::Disconnected(client_id));
            }
            ManagerNotify::ConnectionInterrupt(addr) => {
                println!("Connection with {addr} has been interrupted!");
                let client_id = client_manager.inactivate_client(addr);
                let _ = con_event_sender.send(ConnectionEvent::Interrupt(addr));
                if let Some(game_id) = game_manager.get_game_id(client_id) {
                    match client_id == game_manager.game_host(game_id) {
                        true => {
                            game_manager.remove_game(client_id);
                            let _ = client_event.send(EventBroadcast::GameDeletion(game_id));
                        }
                        false => {
                            game_manager.remove_client_from_game(client_id);
                            let _ = client_event.send(EventBroadcast::GameExit(client_id));
                        }
                    }
                }
                let _ = client_event.send(EventBroadcast::ConnectionInterrupt(client_id));
            }
            ManagerNotify::Message { client_id, content } => {
                println!("{} (#{client_id}): {content}", client_manager.get_client(client_id).name);
                let _ = client_event.send(EventBroadcast::Message { client_id, content });
            }
            ManagerNotify::GameCreation {mut game, host_addr} => {
                println!("{} (#{}) wants to create a game {}", game.host_id, client_manager.get_client(game.host_id).name, game.game_name);
                if game_manager.add_game(&mut game) {
                    let _ = client_event.send(EventBroadcast::GameCreation {game, host_addr});
                }
            }
            ManagerNotify::GameDeletion(host_id) => {
                println!("{} (#{host_id}) wants to delete his game", client_manager.get_client(host_id).name);
                let game_id = game_manager.remove_game(host_id);
                let _ = client_event.send(EventBroadcast::GameDeletion(game_id.unwrap()));
            }
            ManagerNotify::GameEntry { client_id, client_addr, game_id, password } => {
                println!("{} (#{client_id}) wants to join the game #{game_id} with password: {password:?}", client_manager.get_client(client_id).name);
                game_manager.add_client_to_game(client_id, game_id);
                let _ = client_event.send(EventBroadcast::GameEntry { client_id, client_addr, game_id });
            }
            ManagerNotify::GameExit(client_id) => {
                println!("{} (#{client_id}) wants to leave his game", client_manager.get_client(client_id).name);
                game_manager.remove_client_from_game(client_id);
                let _ = client_event.send(EventBroadcast::GameExit(client_id));
            }
            ManagerNotify::GameWorld { client_id, scene } => {
                println!("{} (#{client_id}) shares his game world", client_manager.get_client(client_id).name);
                let _ = client_event.send(EventBroadcast::GameWorld { client_id, scene });
            }
            ManagerNotify::Command { response, value } => {
                let _ = response.send(match value.as_str() {
                    "help" => "help - print this help\nclients - list connected clients\ngames - list hosted games".to_string(),
                    "clients" => client_manager.get_clients().to_string(),
                    "games" => game_manager.get_games().to_string(),
                    _ => format!("'{value}' is not a valid command, try 'help' instead")
                });
            }
        }
        let _ = client_list.send(client_manager.get_clients());
        let _ = game_list.send(game_manager.get_games());
    }
}

pub enum ConnectionEvent {
    Interrupt(IpAddr),
    Reconnect(IpAddr)
}

const MAX_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(60);

// Track all inactive clients (those without a connection) and formally disconnect them after 60
// seconds without reconnect
pub async fn disconnect_timeout_handler(
    sender: UnboundedSender<ManagerNotify>,
    mut receiver: UnboundedReceiver<ConnectionEvent>
) {
    let mut clients: HashMap<IpAddr, Instant> = HashMap::new();
    loop {
        let oldest = clients.iter().min();
        tokio::select! {
            _ = sleep_until(*oldest.map(|(_, instant)| instant).unwrap_or(&(Instant::now() + MAX_DISCONNECT_TIMEOUT))) => {
                if let Some((addr, _)) = oldest {
                    let _ = sender.send(ManagerNotify::Disconnected(*addr));
                    clients.remove(&addr.clone());
                }
            }
            Some(event) = receiver.recv() => {
                match event {
                    ConnectionEvent::Interrupt(addr) => {
                        clients.insert(addr, Instant::now() + MAX_DISCONNECT_TIMEOUT);
                    }
                    ConnectionEvent::Reconnect(addr) => {
                        clients.remove(&addr);
                    }
                }
            }
        }
    }
}
