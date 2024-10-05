use std::net::{IpAddr, SocketAddr, UdpSocket};
use bevy_utils::HashMap;
use manager::{client_game_manager, ManagerNotify};
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream, ToSocketAddrs}, sync::{broadcast::{self, Receiver}, mpsc::{unbounded_channel, UnboundedSender}}};

use crate::{Client, ClientStatus, Game, GameUpdateData, Lobby, LobbyConnectionAcceptResponse, LobbyUpdateData, PackageType};

mod manager;

#[derive(Clone)]
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
}

pub async fn listen<A: ToSocketAddrs>(tcp_addr: A) -> std::io::Result<()> {
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
) -> tokio::io::Result<()> {
    let mut buf = [0; 1];
    let _ = tcp.read(&mut buf).await?;
    let client_id;
    match PackageType::from(buf[0]) {
        PackageType::LobbyConnect => {
            // name length
            let mut buf = [0; 1];
            let _ = tcp.read(&mut buf).await?;
            // name
            let mut buf = vec![0; buf[0].into()];
            let _ = tcp.read(&mut buf).await?;
            let name = String::from_utf8_lossy(&buf).to_string();
            println!("{addr} requested a connection; name: {}", name);
            let _ = sender.send(ManagerNotify::Connected { addr: addr.ip(), client: Client::new(name) });
            tcp.writable().await?;
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
                            tcp.try_write(&[u8::from(PackageType::LobbyConnectionDeny)])?;
                            return Ok(());
                        } else {continue;}
                    }
                    _ => {}
                }
            };
            let clients = client_list.recv().await.unwrap();
            let games = game_list.recv().await.unwrap();
            let response = Vec::from(LobbyConnectionAcceptResponse {
                client_id,
                lobby: Lobby {
                    client_count: clients.len() as u16,
                    game_count: 0,
                    clients,
                    games,
                }
            });
            tcp.try_write(response.as_slice())?;
        }
        _ => {return Ok(());}
    }
    loop {
        let mut buf = [0; 1];
        tokio::select! {
            Ok(n) = tcp.read(&mut buf) => {
                if n == 0 {
                    let _ = sender.send(ManagerNotify::ConnectionInterrupt(addr.ip()));
                    break;
                }
                match PackageType::from(buf[0]) {
                    PackageType::LobbyDisconnect => {
                        println!("{addr} requested a disconnect");
                        let _ = sender.send(ManagerNotify::Disconnected(addr.ip()));
                        return Ok(());
                    }
                    PackageType::LobbyUpdate(crate::LobbyUpdate::Message) => {
                        println!("{addr} has send a message");
                        let _ = tcp.read(&mut buf).await;
                        let mut msg = vec![0; buf[0].into()];
                        let _ = tcp.read(&mut msg).await;
                        let _ = sender.send(ManagerNotify::Message {client_id, content: String::from_utf8_lossy(&msg).into()});
                    }
                    PackageType::GameCreation => {
                        let _ = tcp.read(&mut buf).await;
                        let with_password = match buf[0] {
                            1 => true,
                            _ => false,
                        };
                        let _ = tcp.read(&mut buf).await;
                        let mut name = vec![0; buf[0].into()];
                        let _ = tcp.read(&mut name).await;
                        let _ = sender.send(ManagerNotify::GameCreation(Game {
                            game_id: 0,
                            host_id: client_id,
                            password: with_password,
                            game_name: String::from_utf8_lossy(&name).into(),
                            clients: vec![client_id],
                        }));
                    }
                    PackageType::GameDeletion => {
                        let _ = sender.send(ManagerNotify::GameDeletion(client_id));
                    }
                    PackageType::GameEntry => {
                        let mut game_id = [0; 2];
                        let _ = tcp.read(&mut game_id).await;
                        let _ = sender.send(ManagerNotify::GameEntry { client_id, game_id: u16::from_ne_bytes(game_id) });
                    }
                    PackageType::GameExit => {
                        let _ = sender.send(ManagerNotify::GameExit(client_id));
                    }
                    _ => println!("unknown data received... buf: {buf:?}"),
                }
            }
            Ok(event) = client_event.recv() => {
                match event {
                    EventBroadcast::Connected{client, ..} => {
                        LobbyUpdateData::Connect(client).write(&mut tcp).await?;
                    }
                    EventBroadcast::Disconnected(client_id) => {
                        LobbyUpdateData::Disconnect(client_id).write(&mut tcp).await?;
                    }
                    EventBroadcast::ConnectionInterrupt(client_id) => {
                        LobbyUpdateData::ConnectionInterrupt(client_id).write(&mut tcp).await?;
                    }
                    EventBroadcast::Reconnected{client, ..} => {
                        LobbyUpdateData::Reconnect(client.client_id).write(&mut tcp).await?;
                    }
                    EventBroadcast::Message {client_id, content} => {
                        LobbyUpdateData::Message {sender: client_id, length: content.len() as u8, content}.write(&mut tcp).await?;
                    }
                    EventBroadcast::GameCreation(game) => {
                        GameUpdateData::Creation(game).write(&mut tcp).await?;
                    }
                    EventBroadcast::GameDeletion(host_id) => {
                        GameUpdateData::Deletion(host_id).write(&mut tcp).await?;
                    }
                    EventBroadcast::GameEntry { client_id, game_id } => {
                        GameUpdateData::Entry { client_id, game_id }.write(&mut tcp).await?;
                    }
                    EventBroadcast::GameExit(client_id) => {
                        GameUpdateData::Exit(client_id).write(&mut tcp).await?;
                    }
                    _ => {}
                }
            }
        };
    }
    Ok(())
}

impl From<LobbyConnectionAcceptResponse> for Vec<u8> {
    fn from(response: LobbyConnectionAcceptResponse) -> Self {
        let mut bytes: Vec<u8> = vec![];
        bytes.push(u8::from(PackageType::LobbyConnectionAccept));
        bytes.extend_from_slice(&response.client_id.to_ne_bytes());
        bytes.extend_from_slice(&response.lobby.game_count.to_ne_bytes());
        bytes.extend_from_slice(&response.lobby.client_count.to_ne_bytes());
        for (_, game) in response.lobby.games {
            bytes.extend_from_slice(&Vec::from(game));
        }
        for (_, client) in response.lobby.clients {
            bytes.extend_from_slice(&Vec::from(client));
        }
        bytes
    }
}

impl From<Client> for Vec<u8> {
    fn from(client: Client) -> Self {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(&client.client_id.to_ne_bytes());
        bytes.push(match client.in_game {
            true => 1,
            false => 0,
        });
        bytes.push(client.name.len() as u8);
        match client.status {
            ClientStatus::Active => bytes.push(1),
            ClientStatus::Idle(duration) => {
                bytes.extend_from_slice(&[
                    0,
                    duration.as_secs() as u8
                ]);
            }
        }
        bytes.extend_from_slice(client.name.as_bytes());
        bytes
    }
}

impl From<Game> for Vec<u8> {
    fn from(game: Game) -> Self {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(&game.game_id.to_ne_bytes());
        bytes.extend_from_slice(&game.host_id.to_ne_bytes());
        bytes.push(match game.password {
            true => 1,
            false => 0,
        });
        bytes.push(game.game_name.len() as u8);
        bytes.extend_from_slice(game.game_name.as_bytes());
        bytes.push(game.clients.len() as u8);
        for client_id in game.clients.iter() {
            bytes.extend_from_slice(&client_id.to_ne_bytes());
        }
        bytes
    }
}
