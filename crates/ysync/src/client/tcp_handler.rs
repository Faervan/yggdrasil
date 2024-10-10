use crossbeam::channel::Sender;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, select, sync::mpsc::UnboundedReceiver};

use crate::{Client, Game, GameUpdate, GameUpdateData, LobbyUpdate, LobbyUpdateData, PackageType};

use super::{TcpPackage, TcpUpdate};



pub async fn tcp_handler(mut tcp: TcpStream, mut receiver: UnboundedReceiver<TcpPackage>, sender: Sender<TcpUpdate>) {
    loop {
        let mut buf = [0; 1];
        select! {
            _ = tcp.read(&mut buf) => {
                println!("tcp_handler received: {}, PackageType: {:?}", buf[0], PackageType::from(buf[0]));
                match PackageType::from(buf[0]) {
                    PackageType::LobbyUpdate(LobbyUpdate::Connect) => {
                        let client = Client::from(&mut tcp).await;
                        println!("A client connected! {client:#?}");
                        let _ = sender.send(TcpUpdate::LobbyUpdate(LobbyUpdateData::Connect(client)));
                    }
                    PackageType::LobbyUpdate(with_id) => {
                        let mut id = [0; 2];
                        let _ = tcp.read(&mut id).await;
                        let client_id = u16::from_ne_bytes(id);
                        match with_id {
                            LobbyUpdate::Disconnect => {
                                println!("client with id {client_id} disconnected");
                                let _ = sender.send(TcpUpdate::LobbyUpdate(LobbyUpdateData::Disconnect(client_id)));
                            }
                            LobbyUpdate::ConnectionInterrupt => {
                                println!("connection to client {client_id} was interrupted");
                                let _ = sender.send(TcpUpdate::LobbyUpdate(LobbyUpdateData::ConnectionInterrupt(client_id)));
                            }
                            LobbyUpdate::Reconnect => {
                                println!("client with id {client_id} reconnected");
                                let _ = sender.send(TcpUpdate::LobbyUpdate(LobbyUpdateData::Reconnect(client_id)));
                            }
                            LobbyUpdate::Message => {
                                let _ = tcp.read(&mut buf).await;
                                let mut msg = vec![0; buf[0].into()];
                                let _ = tcp.read(&mut msg).await;
                                let _ = sender.send(TcpUpdate::LobbyUpdate(LobbyUpdateData::Message {sender: client_id, length: buf[0], content: String::from_utf8_lossy(&msg).to_string()}));
                            }
                            _ => {println!("got lobbyupdate....wait what?!")}
                        }
                    }
                    PackageType::GameUpdate(GameUpdate::Creation) => {
                        let game = Game::from(&mut tcp).await;
                        println!("A game got created! {game:#?}");
                        let _ = sender.send(TcpUpdate::GameUpdate(GameUpdateData::Creation(game)));
                    }
                    PackageType::GameUpdate(GameUpdate::Deletion) => {
                        let mut id_buf = [0; 2];
                        let _ = tcp.read(&mut id_buf);
                        let game_id = u16::from_ne_bytes(id_buf);
                        let _ = sender.send(TcpUpdate::GameUpdate(GameUpdateData::Deletion(game_id)));
                    }
                    PackageType::GameUpdate(GameUpdate::Entry) => {
                        println!("got GameUpdate::Entry...");
                        let mut id_buf = [0; 2];
                        let _ = tcp.read(&mut id_buf);
                        let client_id = u16::from_ne_bytes(id_buf);
                        let _ = tcp.read(&mut id_buf);
                        let game_id = u16::from_ne_bytes(id_buf);
                        let _ = sender.send(TcpUpdate::GameUpdate(GameUpdateData::Entry { client_id, game_id }));
                        println!("done receiving GameUpdate::Entry");
                    }
                    PackageType::GameUpdate(GameUpdate::Exit) => {
                        let mut id_buf = [0; 2];
                        let _ = tcp.read(&mut id_buf);
                        let client_id = u16::from_ne_bytes(id_buf);
                        let _ = sender.send(TcpUpdate::GameUpdate(GameUpdateData::Exit(client_id)));
                    }
                    PackageType::GameWorld => {
                        println!("client got GameWorld PackageType...");
                        let mut len_buf = [0;2];
                        let _ = tcp.read(&mut len_buf).await;
                        let mut serialized_scene = vec![0; u16::from_ne_bytes(len_buf).into()];
                        let _ = tcp.read(&mut serialized_scene).await;
                        let _ = sender.send(TcpUpdate::GameWorld(String::from_utf8_lossy(&serialized_scene).to_string()));
                        println!("got all data, send TcpUpdate::GameWorld into channel...");
                    }
                    _ => {
                        println!("unknown data received: {}", buf[0]);
                    }
                }
            }
            Some(event) = receiver.recv() => {
                println!("Client is sending TcpPackage: {event:?}");
                match event {
                    TcpPackage::Disconnect => {
                        let _ = tcp.write(&[u8::from(PackageType::LobbyDisconnect)]).await;
                    }
                    TcpPackage::Message(msg) => {
                        let mut bytes = vec![];
                        bytes.push(u8::from(PackageType::LobbyUpdate(crate::LobbyUpdate::Message)));
                        bytes.push(msg.len() as u8);
                        bytes.extend_from_slice(msg.as_bytes());
                        let _ = tcp.write(bytes.as_slice()).await;
                    }
                    TcpPackage::GameCreation { name, with_password } => {
                        let mut bytes = vec![];
                        bytes.push(u8::from(PackageType::GameCreation));
                        bytes.push(match with_password {
                            true => 1,
                            false => 0,
                        });
                        bytes.push(name.len() as u8);
                        bytes.extend_from_slice(name.as_bytes());
                        let _ = tcp.write(bytes.as_slice()).await;
                    }
                    TcpPackage::GameDeletion => {
                        let _ = tcp.write(&[u8::from(PackageType::GameDeletion)]).await;
                    }
                    TcpPackage::GameEntry(game_id) => {
                        let mut bytes = vec![];
                        bytes.push(u8::from(PackageType::GameEntry));
                        bytes.extend_from_slice(&game_id.to_ne_bytes());
                        let _ = tcp.write(bytes.as_slice()).await;
                    }
                    TcpPackage::GameExit => {
                        let _ = tcp.write(&[u8::from(PackageType::GameExit)]).await;
                    }
                    TcpPackage::GameWorld(serialized_scene) => {
                        let mut bytes = vec![];
                        bytes.push(u8::from(PackageType::GameWorld));
                        bytes.extend_from_slice(&(serialized_scene.len() as u16).to_ne_bytes());
                        bytes.extend_from_slice(&serialized_scene.as_bytes());
                        let _ = tcp.write(bytes.as_slice()).await;
                    }
                }
            }
        }
    }
}
