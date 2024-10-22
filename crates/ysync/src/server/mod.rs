use std::net::IpAddr;
use manager::client_game_manager;
use tcp_handler::handle_client_tcp;
use tokio::{net::{TcpListener, ToSocketAddrs}, sync::{broadcast, mpsc::unbounded_channel}};
use udp_handler::udp_handler;

use crate::{Client, Game};

mod manager;
mod tcp_handler;
mod udp_handler;

#[derive(Clone, Debug)]
enum EventBroadcast {
    Connected {
        addr: IpAddr,
        client: Client,
    },
    Disconnected(u16),
    ConnectionInterrupt(u16),
    Reconnected {
        addr: IpAddr,
        client: Client,
    },
    Multiconnect(IpAddr),
    Message {
        client_id: u16,
        content: String
    },
    GameCreation {
        game: Game,
        host_addr: IpAddr
    },
    GameDeletion(/*game_id:*/u16),
    GameEntry {
        client_id: u16,
        client_addr: IpAddr,
        game_id: u16,
    },
    GameExit(/*client_id*/u16),
    GameWorld {
        client_id: u16,
        scene: String,
    },
}

pub async fn listen<A: ToSocketAddrs>(tcp_addr: A, debug_state: Option<()>) -> std::io::Result<()> {
    // Channel to send data to the client manager
    let (client_send, manager_recv) = unbounded_channel();
    // Channel for client join/leave events
    let (client_event_channel, _) = broadcast::channel(5);
    // Channel for client_list broadcast
    let (client_list_channel, _) = broadcast::channel(1);
    // Channel for game_list broadcast
    let (game_list_channel, _) = broadcast::channel(1);
    let listener = TcpListener::bind(tcp_addr).await?;
    tokio::spawn(client_game_manager(
        client_event_channel.clone(),
        client_list_channel.clone(),
        game_list_channel.clone(),
        manager_recv,
    ));
    tokio::spawn(udp_handler(client_event_channel.subscribe()));
    loop {
        let (tcp, addr) = listener.accept().await?;
        tokio::spawn(handle_client_tcp(
            tcp,
            addr,
            client_send.clone(),
            client_event_channel.subscribe(),
            client_list_channel.subscribe(),
            game_list_channel.subscribe(),
            debug_state,
        ));
    }
}
