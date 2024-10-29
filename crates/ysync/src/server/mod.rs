use std::net::IpAddr;
use manager::{client_game_manager, disconnect_timeout_handler, ManagerNotify};
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

pub async fn listen<A: ToSocketAddrs>(tcp_addr: A, rcon: Option<(u16, String)>) -> std::io::Result<()> {
    // Channel to send data to the client manager
    let (client_send, manager_recv) = unbounded_channel();
    // Channel for client join/leave events
    let (client_event_channel, _) = broadcast::channel(5);
    // Channel for client_list broadcast
    let (client_list_channel, _) = broadcast::channel(1);
    // Channel for game_list broadcast
    let (game_list_channel, _) = broadcast::channel(1);
    // Channel for connection events
    let (con_event_send, con_event_recv) = unbounded_channel();

    let listener = TcpListener::bind(tcp_addr).await?;
    tokio::spawn(client_game_manager(
        client_event_channel.clone(),
        client_list_channel.clone(),
        game_list_channel.clone(),
        manager_recv,
        con_event_send,
    ));
    tokio::spawn(disconnect_timeout_handler(client_send.clone(), con_event_recv));
    if let Some((port, password)) = rcon {
        let (s, mut r) = unbounded_channel();
        tokio::spawn(rcon_server::listen(Some(port), password, s));
        let manager_notify = client_send.clone();
        tokio::spawn(async move {
            loop {
                match r.recv().await {
                    Some((sx, command)) => {
                        let _ = manager_notify.send(ManagerNotify::Command { response: sx, value: command });
                    }
                    None => {return}
                }
            }
        });
    }
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
        ));
    }
}
