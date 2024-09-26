use std::{collections::HashMap, fmt, io::{Read, Write}, net::{TcpStream, ToSocketAddrs, UdpSocket}};

use crate::{
    PackageType,
    Lobby,
    LobbyConnectionAcceptResponse,
    Client,
};

#[derive(Debug)]
pub struct ConnectionSocket {
    //id of the game connected to
    game_id: Option<u16>,
    //id of the player
    client_id: u16,
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
}

impl From<&mut TcpStream> for Client {
    fn from(tcp: &mut TcpStream) -> Self {
        let mut buf = [0; 4];
        let _ = tcp.read(&mut buf);
        let client_id = u16::from_ne_bytes(buf[..2].try_into().unwrap());
        let in_game = match buf[2] {
            1 => true,
            _ => false,
        };
        let mut name = vec![0; buf[3].into()];
        let _ = tcp.read(&mut name);
        Client {
            client_id,
            in_game,
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
    pub fn build<A: ToSocketAddrs>(lobby_addr: A, sender_name: String) -> Result<(ConnectionSocket, Lobby), LobbyConnectionError> {
        let mut tcp = TcpStream::connect(&lobby_addr)?;
        let udp = UdpSocket::bind(lobby_addr)?;
        let mut package: Vec<u8> = vec![];
        package.push(u8::from(PackageType::LobbyConnection));
        package.extend_from_slice(sender_name.as_bytes());
        tcp.write(&package)?;
        let mut buf = [0; 7];
        tcp.read(&mut buf)?;
        match PackageType::from(buf[0])  {
            PackageType::ConnectionAccept => {},
            PackageType::ConnectionDeny => return Err(LobbyConnectionError(LobbyConnectionErrorReason::ConnectionDenied)),
            _ => return Err(LobbyConnectionError(LobbyConnectionErrorReason::InvalidResponse)),
        }
        let mut response = LobbyConnectionAcceptResponse::from(&buf[1..]);
        for _ in 0..response.lobby.client_count {
            let client = Client::from(&mut tcp);
            response.lobby.clients.insert(client.client_id, client);
        }
        Ok((
            ConnectionSocket {
                game_id: None,
                client_id: response.client_id,
                tcp_stream: tcp,
                udp_socket: udp,
            },
            response.lobby,
        ))
    }
}

