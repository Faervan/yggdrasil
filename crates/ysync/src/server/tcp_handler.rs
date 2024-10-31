use std::{net::SocketAddr, time::{Duration, Instant}};

use bevy_utils::HashMap;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::{broadcast::Receiver, mpsc::UnboundedSender}, time::sleep};

use crate::{Client, Game, GameUpdate, Lobby, LobbyConnectionDenyReason, LobbyConnectionRequest, LobbyConnectionResponse, LobbyUpdate, TcpFromClient, TcpFromServer};

use super::{manager::ManagerNotify, EventBroadcast};

pub async fn handle_client_tcp(
    mut tcp: TcpStream,
    addr: SocketAddr,
    sender: UnboundedSender<ManagerNotify>,
    mut client_event: Receiver<EventBroadcast>,
    mut client_list: Receiver<HashMap<u16, Client>>,
    mut game_list: Receiver<HashMap<u16, Game>>,
) -> tokio::io::Result<()> {
    let client_id;
    let mut buf = [0; 4];
    tcp.read(&mut buf).await?;
    let pkg_len = u32::from_ne_bytes(buf) as usize;
    let mut pkg_buf = vec![0; pkg_len];
    tcp.read(&mut pkg_buf).await?;
    match LobbyConnectionRequest::from_buf(&pkg_buf) {
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
    let mut last_connection = Instant::now();
    const MAX_TIMEOUT: Duration = Duration::from_secs(6);
    loop {
        let mut buf = [0; 4];
        tokio::select! {
            Ok(n) = tcp.read(&mut buf) => {
                if n == 0 {
                    let _ = sender.send(ManagerNotify::ConnectionInterrupt(addr.ip()));
                    break;
                }
                let pkg_len = u32::from_ne_bytes(buf) as usize;
                let mut pkg_buf = vec![0; pkg_len];
                let mut bytes_read = 0;
                loop {
                    let n = tcp.read(&mut pkg_buf[bytes_read..]).await;
                    match n {
                        Ok(len) => {
                            bytes_read += len;
                        }
                        Err(e) => {
                            println!("There was an error {e}");
                            continue;
                        }
                    }
                    if bytes_read >= pkg_len {
                        if bytes_read > pkg_len {
                            println!("\nShit...read more bytes ({bytes_read}) than length of pkg ({pkg_len})\n")
                        }
                        break;
                    }
                }
                let package;
                match TcpFromClient::from_buf(&pkg_buf) {
                    Ok(pkg) => package = pkg,
                    Err(e) => {
                        println!("Received invalid package from {addr} (#{client_id}), e: {e}\n\tbuf: {buf:?}");
                        continue;
                    }
                }
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
                        let _ = sender.send(ManagerNotify::GameCreation {
                            game: Game {
                                game_id: 0,
                                host_id: client_id,
                                password: password,
                                game_name: name,
                                clients: vec![client_id],
                            },
                            host_addr: addr.ip()
                        });
                    }
                    TcpFromClient::GameDeletion => {
                        let _ = sender.send(ManagerNotify::GameDeletion(client_id));
                    }
                    TcpFromClient::GameEntry { password, game_id } => {
                        let _ = sender.send(ManagerNotify::GameEntry { password, client_id, client_addr: addr.ip(), game_id });
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
                    TcpFromClient::Heartbeat => last_connection = Instant::now(),
                }
            }
            Ok(event) = client_event.recv() => {
                if last_connection.elapsed() >= MAX_TIMEOUT {
                    let _ = sender.send(ManagerNotify::ConnectionInterrupt(addr.ip()));
                    break;
                }
                let pkg = match event {
                    EventBroadcast::Connected{client, ..} => {
                        TcpFromServer::LobbyUpdate(LobbyUpdate::Connection(client))
                    }
                    EventBroadcast::Disconnected(client_id) => {
                        TcpFromServer::LobbyUpdate(LobbyUpdate::Disconnection(client_id))
                    }
                    EventBroadcast::ConnectionInterrupt(client_id) => {
                        TcpFromServer::LobbyUpdate(LobbyUpdate::ConnectionInterrupt(client_id))
                    }
                    EventBroadcast::Reconnected{client, ..} => {
                        TcpFromServer::LobbyUpdate(LobbyUpdate::Reconnect(client.client_id))
                    }
                    EventBroadcast::Message {client_id, content} => {
                        TcpFromServer::LobbyUpdate(LobbyUpdate::Message { sender: client_id, content })
                    }
                    EventBroadcast::GameCreation {game, ..} => {
                        TcpFromServer::GameUpdate(GameUpdate::Creation(game))
                    }
                    EventBroadcast::GameDeletion(host_id) => {
                        TcpFromServer::GameUpdate(GameUpdate::Deletion(host_id))
                    }
                    EventBroadcast::GameEntry { client_id, game_id, .. } => {
                        TcpFromServer::GameUpdate(GameUpdate::Entry { client_id, game_id })
                    }
                    EventBroadcast::GameExit(client_id) => {
                        TcpFromServer::GameUpdate(GameUpdate::Exit(client_id))
                    }
                    EventBroadcast::GameWorld { client_id: sender, scene } => {
                        if client_id != sender {
                            println!("got GameWorld EventBroadcast...\n\tclient_id: {client_id}\n\tsender: {sender}");
                            let pkg = TcpFromServer::GameUpdate(GameUpdate::World(scene));
                            pkg
                        } else {continue;}
                    }
                    EventBroadcast::Multiconnect(_) => {continue;}
                }.as_bytes();
                tcp.write(&pkg).await?;
            }
            _ = sleep(MAX_TIMEOUT) => {
                let _ = sender.send(ManagerNotify::ConnectionInterrupt(addr.ip()));
                break;
            }
        };
    }
    Ok(())
}
