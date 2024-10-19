use std::net::{IpAddr, SocketAddr};
use bevy_utils::HashMap;
use manager::{client_game_manager, ManagerNotify};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, ToSocketAddrs}, sync::{broadcast::{self, Receiver}, mpsc::{unbounded_channel, UnboundedSender}}};

use crate::{Client, Game, GameUpdate, Lobby, LobbyConnectionDenyReason, LobbyConnectionRequest, LobbyConnectionResponse, LobbyUpdate, TcpFromClient, TcpFromServer};

mod manager;

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
    GameCreation(Game),
    GameDeletion(/*game_id:*/u16),
    GameEntry {
        client_id: u16,
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
        manager_recv
    ));
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

async fn handle_client_tcp(
    mut tcp: TcpStream,
    addr: SocketAddr,
    sender: UnboundedSender<ManagerNotify>,
    mut client_event: Receiver<EventBroadcast>,
    mut client_list: Receiver<HashMap<u16, Client>>,
    mut game_list: Receiver<HashMap<u16, Game>>,
    debug_state: Option<()>,
) -> tokio::io::Result<()> {
    let client_id;
    let mut buf = [0; LobbyConnectionRequest::MAX_SIZE];
    tcp.read(&mut buf).await?;
    match LobbyConnectionRequest::from_buf(&buf) {
        Ok(LobbyConnectionRequest(name)) => {
            println!("{addr} requested a connection; name: {}", name);
            let _ = sender.send(ManagerNotify::Connected { addr: addr.ip(), client: Client::new(name) });
            client_id = loop {
                match client_event.recv().await.unwrap() {
                    EventBroadcast::Connected {addr: event_addr, client} => {
                        if event_addr == addr.ip() {break client.client_id;} else {continue;}
                    }
                    EventBroadcast::Reconnected {addr: event_addr, client} => {
                        if event_addr == addr.ip() {break client.client_id;} else {continue;}
                    }
                    EventBroadcast::Multiconnect(event_addr) => {
                        if event_addr == addr.ip() {
                            tcp.write(&LobbyConnectionResponse::Deny(LobbyConnectionDenyReason::AlreadyConnected).as_bytes()).await?;
                            return Ok(());
                        } else {continue;}
                    }
                    _ => {}
                }
            };
            let clients = client_list.recv().await.unwrap();
            let games = game_list.recv().await.unwrap();
            let response = LobbyConnectionResponse::Accept { client_id, lobby: Lobby {
                client_count: clients.len() as u16,
                game_count: games.len() as u16,
                clients,
                games
            } }.as_bytes();
            tcp.write(&response).await?;
        }
        Err(e) => {
            println!("Some Client tried to connect with invalid data: e: {e}");
            return Ok(());
        }
    }
    loop {
        let mut buf = [0; 1];
        tokio::select! {
            Ok(n) = tcp.read(&mut buf) => {
                if n == 0 {
                    let _ = sender.send(ManagerNotify::ConnectionInterrupt(addr.ip()));
                    break;
                }
                let mut pkg_buf = [0; TcpFromClient::MAX_SIZE-1];
                let _ = tcp.read(&mut pkg_buf).await;
                let mut combi_buf = buf.to_vec();
                combi_buf.extend_from_slice(&pkg_buf);
                let package;
                match TcpFromClient::from_buf(&combi_buf) {
                    Ok(pkg) => package = pkg,
                    Err(e) => {
                        println!("Received invalid package from {addr} (#{client_id}), e: {e}\n\tbuf: {buf:?}");
                        continue;
                    }
                }
                println!("Server received package from #{client_id}: {package:#?}");
                match package {
                    TcpFromClient::LobbyDisconnect => {
                        println!("{addr} requested a disconnect");
                        let _ = sender.send(ManagerNotify::Disconnected(addr.ip()));
                        return Ok(());
                    }
                    TcpFromClient::Message(content) => {
                        println!("{addr} (#{client_id}) has send a message: {content}");
                        let _ = sender.send(ManagerNotify::Message {client_id, content});
                    }
                    TcpFromClient::GameCreation { password, name } => {
                        let _ = sender.send(ManagerNotify::GameCreation(Game {
                            game_id: 0,
                            host_id: client_id,
                            password: password,
                            game_name: name,
                            clients: vec![client_id],
                        }));
                    }
                    TcpFromClient::GameDeletion => {
                        let _ = sender.send(ManagerNotify::GameDeletion(client_id));
                    }
                    TcpFromClient::GameEntry { password, game_id } => {
                        let _ = sender.send(ManagerNotify::GameEntry { password, client_id, game_id });
                    }
                    TcpFromClient::GameExit => {
                        let _ = sender.send(ManagerNotify::GameExit(client_id));
                    }
                    TcpFromClient::GameWorld(scene) => {
                        let _ = sender.send(ManagerNotify::GameWorld {
                            client_id,
                            scene
                        });
                    }
                }
            }
            Ok(event) = client_event.recv() => {
                println!("Server is sending package: {event:?}");
                match event {
                    EventBroadcast::Connected{client, ..} => {
                        tcp.write(&TcpFromServer::LobbyUpdate(LobbyUpdate::Connection(client)).as_bytes()).await?;
                    }
                    EventBroadcast::Disconnected(client_id) => {
                        tcp.write(&TcpFromServer::LobbyUpdate(LobbyUpdate::Disconnection(client_id)).as_bytes()).await?;
                    }
                    EventBroadcast::ConnectionInterrupt(client_id) => {
                        tcp.write(&TcpFromServer::LobbyUpdate(LobbyUpdate::ConnectionInterrupt(client_id)).as_bytes()).await?;
                    }
                    EventBroadcast::Reconnected{client, ..} => {
                        tcp.write(&TcpFromServer::LobbyUpdate(LobbyUpdate::Reconnect(client.client_id)).as_bytes()).await?;
                    }
                    EventBroadcast::Message {client_id, content} => {
                        tcp.write(&TcpFromServer::LobbyUpdate(LobbyUpdate::Message { sender: client_id, content }).as_bytes()).await?;
                    }
                    EventBroadcast::GameCreation(game) => {
                        tcp.write(&TcpFromServer::GameUpdate(GameUpdate::Creation(game)).as_bytes()).await?;
                    }
                    EventBroadcast::GameDeletion(host_id) => {
                        tcp.write(&TcpFromServer::GameUpdate(GameUpdate::Deletion(host_id)).as_bytes()).await?;
                    }
                    EventBroadcast::GameEntry { client_id, game_id } => {
                        tcp.write(&TcpFromServer::GameUpdate(GameUpdate::Entry { client_id, game_id }).as_bytes()).await?;
                    }
                    EventBroadcast::GameExit(client_id) => {
                        tcp.write(&TcpFromServer::GameUpdate(GameUpdate::Exit(client_id)).as_bytes()).await?;
                    }
                    EventBroadcast::GameWorld { client_id: sender, scene } => {
                        println!("got GameWorld EventBroadcast...\n\tclient_id: {client_id}\n\tsender: {sender}");
                        if client_id != sender || debug_state == Some(()) {
                            let pkg = TcpFromServer::GameUpdate(GameUpdate::World(scene));
                            println!("GameWorld pkg: {pkg:#?}");
                            println!("Match! sending...");
                            let n = tcp.write(&pkg.as_bytes()).await?;
                            println!("Done sending {n} bytes");
                        }
                    }
                    _ => {}
                }
            }
        };
    }
    Ok(())
}
