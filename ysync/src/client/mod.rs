use std::{fmt, time::Duration};

use bevy_utils::HashMap;
use crossbeam::channel::{Receiver, Sender};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpStream, ToSocketAddrs, UdpSocket}, select, sync::mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::{
    Client, ClientStatus, Lobby, LobbyConnectionAcceptResponse, LobbyUpdate, LobbyUpdateData, PackageType
};

#[derive(Debug)]
pub struct ConnectionSocket {
    //id of the game connected to
    pub game_id: Option<u16>,
    //id of the player
    pub client_id: u16,
    pub tcp_send: UnboundedSender<TcpPackage>,
    pub tcp_recv: Receiver<LobbyUpdateData>,
    pub udp_socket: UdpSocket,
}

pub enum TcpPackage {
    Disconnect,
    Message(String),
}

impl Client {
    async fn from(tcp: &mut TcpStream) -> Self {
        let mut buf = [0; 5];
        let _ = tcp.read(&mut buf).await;
        println!("received buffer for client: {buf:?}");
        let client_id = u16::from_ne_bytes(buf[..2].try_into().unwrap());
        let in_game = match buf[2] {
            1 => true,
            _ => false,
        };
        let status = match buf[4] {
            1 => {
                ClientStatus::Active
            }
            _ => ClientStatus::Idle({
                let mut seconds = [0, 1];
                let _ = tcp.read(&mut seconds);
                Duration::from_secs(seconds[0] as u64)
            }),
        };
        let mut name = vec![0; buf[3].into()];
        let _ = tcp.read(&mut name).await;
        println!("receivged buffer for client name {name:?}");
        Client {
            client_id,
            in_game,
            status,
            name: String::from_utf8_lossy(&name).to_string(),
        }
    }
}

#[derive(Debug)]
pub struct LobbyConnectionError(LobbyConnectionErrorReason);

#[derive(Debug)]
enum LobbyConnectionErrorReason {
    ConnectionDenied,
    InvalidResponse,
    NetworkError,
    Timeout,
}

impl fmt::Display for LobbyConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyConnectionError(LobbyConnectionErrorReason::ConnectionDenied) => {
                write!(f, "Connection refused! Can't connect to lobby.")
            }
            LobbyConnectionError(LobbyConnectionErrorReason::InvalidResponse) => {
                write!(f, "Got an invalid response from server.")
            }
            LobbyConnectionError(LobbyConnectionErrorReason::NetworkError) => {
                write!(f, "Server unreachable. Check your internet connection.")
            }
            LobbyConnectionError(LobbyConnectionErrorReason::Timeout) => {
                write!(f, "Timeout reached, took too long to connect to lobby.")
            }
        }
    }
}

impl From<std::io::Error> for LobbyConnectionError {
    fn from(err: std::io::Error) -> Self {
        println!("err: {err}");
        LobbyConnectionError(LobbyConnectionErrorReason::NetworkError)
    }
}

impl From<&[u8]> for LobbyConnectionAcceptResponse {
    fn from(bytes: &[u8]) -> Self {
        let client_id = u16::from_ne_bytes(bytes[..2].try_into().unwrap());
        let game_count = u16::from_ne_bytes(bytes[2..4].try_into().unwrap());
        let client_count = u16::from_ne_bytes(bytes[4..6].try_into().unwrap());
        LobbyConnectionAcceptResponse {
            client_id,
            lobby: Lobby {
                game_count,
                client_count,
                games: HashMap::new(),
                clients: HashMap::new(),
            },
        }
    }
}

impl ConnectionSocket {
    pub async fn build<A: ToSocketAddrs + std::fmt::Display>(lobby_addr: A, local_udp_sock: A, sender_name: String) -> Result<(ConnectionSocket, Lobby), LobbyConnectionError> {
        let mut tcp: TcpStream;
        select! {
            tcp_bind = TcpStream::connect(&lobby_addr) => {tcp = tcp_bind?;},
            _ = tokio::time::sleep(Duration::from_secs(5)) => return Err(LobbyConnectionError(LobbyConnectionErrorReason::Timeout)),
        }
        let udp = UdpSocket::bind(local_udp_sock).await?;
        let mut package: Vec<u8> = vec![];
        package.push(u8::from(PackageType::LobbyConnect));
        package.push(sender_name.len() as u8);
        package.extend_from_slice(sender_name.as_bytes());
        tcp.write(&package).await?;
        let mut buf = [0; 7];
        tcp.read(&mut buf).await?;
        match PackageType::from(buf[0])  {
            PackageType::LobbyConnectionAccept => {}
            PackageType::LobbyConnectionDeny => return Err(LobbyConnectionError(LobbyConnectionErrorReason::ConnectionDenied)),
            _ => return Err(LobbyConnectionError(LobbyConnectionErrorReason::InvalidResponse)),
        }
        let mut response = LobbyConnectionAcceptResponse::from(&buf[1..]);
        println!("got resonse: {response:#?}");
        for _ in 0..response.lobby.client_count {
            println!("execute client recieving...");
            let client = Client::from(&mut tcp).await;
            response.lobby.clients.insert(client.client_id, client);
        }
        let (async_out, sync_in) = crossbeam::channel::unbounded();
        let (sync_out, async_in) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(tcp_handler(tcp, async_in, async_out));
        Ok((
            ConnectionSocket {
                game_id: None,
                client_id: response.client_id,
                tcp_send: sync_out,
                tcp_recv: sync_in,
                udp_socket: udp,
            },
            response.lobby,
        ))
    }
}

async fn tcp_handler(mut tcp: TcpStream, mut receiver: UnboundedReceiver<TcpPackage>, sender: Sender<LobbyUpdateData>) {
    loop {
        let mut buf = [0; 1];
        select! {
            _ = tcp.read(&mut buf) => {
                match PackageType::from(buf[0]) {
                    PackageType::LobbyUpdate(LobbyUpdate::Connect) => {
                        let client = Client::from(&mut tcp).await;
                        println!("A client connected! {client:#?}");
                        let _ = sender.send(LobbyUpdateData::Connect(client));
                    }
                    PackageType::LobbyUpdate(with_id) => {
                        let mut buf = [0; 2];
                        let _ = tcp.read(&mut buf).await;
                        let client_id = u16::from_ne_bytes(buf);
                        match with_id {
                            LobbyUpdate::Disconnect => {
                                println!("client with id {client_id} disconnected");
                                let _ = sender.send(LobbyUpdateData::Disconnect(client_id));
                            }
                            LobbyUpdate::ConnectionInterrupt => {
                                println!("connection to client {client_id} was interrupted");
                                let _ = sender.send(LobbyUpdateData::ConnectionInterrupt(client_id));
                            }
                            LobbyUpdate::Reconnect => {
                                println!("client with id {client_id} reconnected");
                                let _ = sender.send(LobbyUpdateData::Reconnect(client_id));
                            }
                            _ => {println!("got lobbyupdate")}
                        }
                    }
                    _ => {}
                }
            }
            Some(event) = receiver.recv() => {
                match event {
                    TcpPackage::Disconnect => {
                        let _ = tcp.write(&[u8::from(PackageType::LobbyDisconnect)]).await;
                    }
                    TcpPackage::Message(msg) => {
                        let _ = tcp.write(&[u8::from(PackageType::LobbyUpdate(crate::LobbyUpdate::Message))]).await;
                    }
                }
            }
        }
    }
}
