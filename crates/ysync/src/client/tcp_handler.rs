use crossbeam::channel::Sender;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, select, sync::mpsc::UnboundedReceiver};

use crate::{GameUpdate, LobbyUpdate, TcpFromClient, TcpFromServer};

use super::TcpUpdate;

pub async fn tcp_handler(mut tcp: TcpStream, mut receiver: UnboundedReceiver<TcpFromClient>, sender: Sender<TcpUpdate>) {
    loop {
        let mut buf = [0; 4];
        select! {
            n = tcp.read(&mut buf) => {
                let pkg_len = u32::from_ne_bytes(buf) as usize;
                if let Ok(0) = n {
                    println!("Lost connection to server!");
                    return;
                }
                let mut pkg_buf = vec![0; pkg_len];
                let mut bytes_read = 0;
                loop {
                    let n = tcp.read(&mut pkg_buf[bytes_read..]).await;
                    match n {
                        Ok(len) => bytes_read += len,
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
                match TcpFromServer::from_buf(&pkg_buf) {
                    Ok(pkg) => package = pkg,
                    Err(e) => {
                        println!("Received invalid package, e: {e}\n\tbuf: {buf:?}");
                        continue;
                    }
                }
                match &package {
                    TcpFromServer::LobbyUpdate(update) => {
                        match update {
                            LobbyUpdate::Connection(client) => {
                                println!("A client connected! {client:#?}");
                            }
                            LobbyUpdate::Disconnection(client_id) => {
                                println!("client with id {client_id} disconnected");
                            }
                            LobbyUpdate::ConnectionInterrupt(client_id) => {
                                println!("connection to Client#{client_id} was interrupted");
                            }
                            LobbyUpdate::Reconnect(client_id) => {
                                println!("client with id {client_id} reconnected");
                            }
                            LobbyUpdate::Message {sender, content} => {
                                println!("client#{sender} has send a message: {content}");
                            }
                            LobbyUpdate::Default => println!("unexpectedly received a LobbyUpdate::Default")
                        }
                        let _ = sender.send(TcpUpdate::LobbyUpdate(update.clone()));
                    }
                    TcpFromServer::GameUpdate(update) => {
                        match update {
                            GameUpdate::Creation(game) => {
                                println!("A game got created! {game:#?}");
                            }
                            GameUpdate::Deletion(game_id) => {
                                println!("Game#{game_id} got deleted!");
                            }
                            GameUpdate::Entry { client_id, game_id } => {
                                println!("Client#{client_id} joined game#{game_id}");
                            }
                            GameUpdate::Exit(client_id) => {
                                println!("Client#{client_id} left the game he was in");
                            }
                            GameUpdate::World(scene) => {
                                println!("Received a scene! {scene}");
                            }
                            GameUpdate::Default => println!("unexpectedly received a GameUpdate::Default")
                        }
                        let _ = sender.send(TcpUpdate::GameUpdate(update.clone()));
                    }
                }
            }
            Some(event) = receiver.recv() => {
                let _ = tcp.write(&event.as_bytes()).await;
            }
        }
    }
}
