use std::{collections::HashMap, net::{SocketAddr, UdpSocket}};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{Client, Lobby, LobbyConnectionAcceptResponse, PackageType};

pub struct ClientConnection {
    client: Client,
    addr: SocketAddr,
}

pub async fn listen<A: ToSocketAddrs>(tcp_addr: A) -> std::io::Result<()> {
    let listener = TcpListener::bind(tcp_addr).await?;
    loop {
        let (tcp, addr) = listener.accept().await?;
        tokio::spawn(handle_client_tcp(tcp, addr));
    }
}

async fn handle_client_tcp(tcp: TcpStream, addr: SocketAddr) -> tokio::io::Result<()> {
    println!("Connection established with {addr:?}");
    let mut pkg = [0; 26];
    tcp.readable().await?;
    let len = tcp.try_read(&mut pkg)?;
    let pkg_type = PackageType::from(pkg[0]);
    let name = String::from_utf8_lossy(&pkg[1..]);
    println!("Received pkg of length {len}:\n\tPackage type: {pkg_type:?}\n\tName: {name}");
    tcp.writable().await?;
    let mut clients = HashMap::new();
    clients.insert(5, Client {client_id: 5, in_game: false, name: "StRIKER19!".to_string()});
    let response = Vec::from(LobbyConnectionAcceptResponse {
        client_id: 7,
        lobby: Lobby {
            client_count: clients.len() as u16,
            game_count: 0,
            clients,
            games: HashMap::new(),
        }
    });
    tcp.try_write(response.as_slice())?;
    Ok(())
}

impl From<LobbyConnectionAcceptResponse> for Vec<u8> {
    fn from(response: LobbyConnectionAcceptResponse) -> Self {
        let mut bytes: Vec<u8> = vec![];
        bytes.push(u8::from(PackageType::ConnectionAccept));
        bytes.extend_from_slice(&response.client_id.to_ne_bytes());
        bytes.extend_from_slice(&response.lobby.game_count.to_ne_bytes());
        bytes.extend_from_slice(&response.lobby.client_count.to_ne_bytes());
        for (k, v) in response.lobby.clients {
            //client_id
            bytes.extend_from_slice(&k.to_ne_bytes());
            //in_game
            bytes.push(0);
            //name_len
            bytes.push(v.name.len() as u8);
            //name
            bytes.extend_from_slice(v.name.as_bytes());
        }
        bytes
    }
}
