use std::{fmt, time::Duration};

use crossbeam::channel::Receiver;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpStream, ToSocketAddrs, UdpSocket}, select, sync::mpsc::UnboundedSender};

use crate::{
    GameUpdate, Lobby, LobbyConnectionDenyReason, LobbyConnectionRequest, LobbyConnectionResponse, LobbyUpdate, TcpFromClient
};

mod tcp_handler;
use tcp_handler::tcp_handler;

#[derive(Debug)]
pub struct ConnectionSocket {
    //id of the game connected to
    pub game_id: Option<u16>,
    //id of the player
    pub client_id: u16,
    pub tcp_send: UnboundedSender<TcpFromClient>,
    pub tcp_recv: Receiver<TcpUpdate>,
    pub udp_socket: UdpSocket,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TcpUpdate {
    LobbyUpdate(LobbyUpdate),
    GameUpdate(GameUpdate),
}

#[derive(Debug)]
pub enum LobbyConnectionError {
    ConnectionDenied(LobbyConnectionDenyReason),
    InvalidResponse,
    NetworkError,
    Timeout,
}

impl fmt::Display for LobbyConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyConnectionError::ConnectionDenied(reason) => {
                write!(f, "Connection refused! Reason: {reason}")
            }
            LobbyConnectionError::InvalidResponse => {
                write!(f, "Got an invalid response from server.")
            }
            LobbyConnectionError::NetworkError => {
                write!(f, "Server unreachable. Check your internet connection.")
            }
            LobbyConnectionError::Timeout => {
                write!(f, "Timeout reached, took too long to connect to lobby.")
            }
        }
    }
}

impl From<std::io::Error> for LobbyConnectionError {
    fn from(err: std::io::Error) -> Self {
        println!("err: {err}");
        LobbyConnectionError::NetworkError
    }
}

impl ConnectionSocket {
    pub async fn build<A: ToSocketAddrs + std::fmt::Display>(lobby_addr: A, local_udp_sock: A, sender_name: String) -> Result<(ConnectionSocket, Lobby), LobbyConnectionError> {
        let mut tcp: TcpStream;
        select! {
            tcp_bind = TcpStream::connect(&lobby_addr) => {tcp = tcp_bind?;},
            _ = tokio::time::sleep(Duration::from_secs(5)) => return Err(LobbyConnectionError::Timeout),
        }
        let udp = UdpSocket::bind(local_udp_sock).await?;
        tcp.write(&LobbyConnectionRequest(sender_name).as_bytes()).await?;
        let mut buf = [0; 4];
        tcp.read(&mut buf).await?;
        let pkg_len = u32::from_ne_bytes(buf) as usize;
        let mut pkg_buf = vec![0; pkg_len];
        tcp.read(&mut pkg_buf).await?;
        let (client_id, lobby) = match LobbyConnectionResponse::from_buf(&pkg_buf)  {
            Ok(LobbyConnectionResponse::Accept { client_id, lobby }) => (client_id, lobby),
            Ok(LobbyConnectionResponse::Deny(reason)) => return Err(LobbyConnectionError::ConnectionDenied(reason)),
            Err(e) => {
                println!("Failed to receive LobbyConnectionResponse, e: {e}");
                return Err(LobbyConnectionError::InvalidResponse)
            },
        };
        let (async_out, sync_in) = crossbeam::channel::unbounded();
        let (sync_out, async_in) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(tcp_handler(tcp, async_in, async_out));
        Ok((
            ConnectionSocket {
                game_id: None,
                client_id,
                tcp_send: sync_out,
                tcp_recv: sync_in,
                udp_socket: udp,
            },
            lobby,
        ))
    }
}
