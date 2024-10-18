use crossbeam::channel::Sender;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, select, sync::mpsc::UnboundedReceiver};

use crate::{GameUpdate, LobbyUpdate, TcpFromClient, TcpFromServer};

use super::TcpUpdate;

pub async fn tcp_handler(mut tcp: TcpStream, mut receiver: UnboundedReceiver<TcpFromClient>, sender: Sender<TcpUpdate>) {
    loop {
        let mut buf = [0; TcpFromServer::MAX_SIZE];
        select! {
            _ = tcp.read(&mut buf) => {
                let package;
                match TcpFromServer::from_buf(&buf) {
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
                                println!("connection to client {client_id} was interrupted");
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
                                println!("A game#{game_id} got deleted!");
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
                println!("Sending TcpPackage: {event:?}");
                let _ = tcp.write(&event.as_bytes()).await;
            }
        }
    }
}
