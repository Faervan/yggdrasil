use std::{collections::VecDeque, net::{IpAddr, SocketAddr, UdpSocket}, time::Instant};
use bevy_utils::HashMap;
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream, ToSocketAddrs}, sync::{broadcast::{self, Receiver, Sender}, mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}}};

use crate::{Client, ClientStatus, Lobby, LobbyConnectionAcceptResponse, LobbyUpdateData, PackageType};

#[derive(Debug, Clone)]
pub struct ClientConnection {
    client: Client,
    active: bool,
    last_con: Instant,
    addr: IpAddr,
}

impl ClientConnection {
    fn as_client(&self) -> Client {
        let mut client = self.client.clone();
        client.status = match self.active {
            true => ClientStatus::Active,
            false => ClientStatus::Idle(self.last_con.elapsed()),
        };
        client
    }
}

#[derive(Debug)]
struct ClientManager {
    clients: Vec<ClientConnection>,
    active_clients: Vec<u16>,
    free_ids: VecDeque<u16>,
}

impl ClientManager {
    fn new() -> ClientManager {
        ClientManager {
            clients: Vec::new(),
            active_clients: Vec::new(),
            free_ids: VecDeque::new(),
        }
    }
    fn add_client(&mut self, mut client: Client, addr: IpAddr) -> Option<(bool, u16)> {
        if let Some(client) = self.clients.iter_mut().find(|c| c.addr == addr) {
            if let Some(_) = self.active_clients.iter().find(|a| **a == client.client.client_id) {
                match client.active {
                    true => return None,
                    false => {
                        client.active = true;
                        return Some((true, client.client.client_id));
                    }
                }
            }
        }
        let mut new_id: bool = false;
        let id = match self.free_ids.pop_front() {
            Some(free_id) => free_id,
            None => {
                new_id = true;
                self.clients.len() as u16
            }
        };
        client.client_id = id;
        match new_id {
            true => self.clients.push(ClientConnection {client, active: true, last_con: Instant::now(), addr}),
            false => self.clients[id as usize] = ClientConnection {client, active: true, last_con: Instant::now(), addr},
        }
        self.active_clients.push(id);
        Some((false, id))
    }
    fn remove_client(&mut self, addr: IpAddr) -> u16 {
        let client_id = self.clients.iter().find(|c| c.addr == addr).unwrap().client.client_id;
        self.active_clients.retain(|a| *a != client_id);
        self.free_ids.push_back(client_id);
        client_id
    }
    fn get_client(&self, client_id: u16) -> Client {
        self.clients[client_id as usize].as_client()
    }
    fn get_clients(&self) -> HashMap<u16, Client> {
        self.active_clients.iter().map(|id| (*id, self.clients[*id as usize].client.clone())).collect()
    }
    fn inactivate_client(&mut self, addr: IpAddr) -> u16 {
        let client = self.clients.iter_mut().find(|c| c.addr == addr).unwrap();
        client.active = false;
        client.last_con = Instant::now();
        client.client.client_id
    }
}

enum ManagerNotify {
    Connected {
        addr: IpAddr,
        client: Client,
    },
    Disconnected(IpAddr),
    ConnectionInterrupt(IpAddr),
    Message(String),
}

#[derive(Clone)]
enum ClientEventBroadcast {
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
    Message(String),
}

async fn client_manager(
    client_event: Sender<ClientEventBroadcast>,
    client_list: Sender<HashMap<u16, Client>>,
    mut receiver: UnboundedReceiver<ManagerNotify>
) -> tokio::io::Result<()> {
    let mut manager = ClientManager::new();
    loop {
        match receiver.recv().await {
            Some(ManagerNotify::Connected { addr, client }) => {
                println!("client connecting! addr: {addr}\n{client:?}");
                if let Some((reconnect, client_id)) = manager.add_client(client, addr) {
                    let client = manager.get_client(client_id);
                    match reconnect {
                        true => {
                            let _ = client_event.send(ClientEventBroadcast::Reconnected {addr, client});
                        }
                        false => {
                            let _ = client_event.send(ClientEventBroadcast::Connected {addr, client});
                        }
                    }
                } else {
                    println!("client is already connected!");
                    let _ = client_event.send(ClientEventBroadcast::Multiconnect(addr));
                }
            }
            Some(ManagerNotify::Disconnected(addr)) => {
                println!("client disconnected! addr: {addr}");
                let client_id = manager.remove_client(addr);
                let _ = client_event.send(ClientEventBroadcast::Disconnected(client_id));
            }
            Some(ManagerNotify::ConnectionInterrupt(addr)) => {
                println!("Connection with {addr} has been interrupted!");
                let client_id = manager.inactivate_client(addr);
                let _ = client_event.send(ClientEventBroadcast::ConnectionInterrupt(client_id));
            }
            _ => println!("shit"),
        }
        let _ = client_list.send(manager.get_clients());
    }
}

pub async fn listen<A: ToSocketAddrs>(tcp_addr: A) -> std::io::Result<()> {
    // Channel to send data to the client manager
    let (client_send, manager_recv) = unbounded_channel();
    // Channel for client join/leave events
    let (client_event_channel, _) = broadcast::channel(5);
    // Channel for client_list broadcast
    let (client_list_channel, _) = broadcast::channel(1);
    let listener = TcpListener::bind(tcp_addr).await?;
    tokio::spawn(client_manager(
        client_event_channel.clone(),
        client_list_channel.clone(),
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
        ));
    }
}

async fn handle_client_tcp(
    mut tcp: TcpStream,
    addr: SocketAddr,
    sender: UnboundedSender<ManagerNotify>,
    mut client_event: Receiver<ClientEventBroadcast>,
    mut client_list: Receiver<HashMap<u16, Client>>,
) -> tokio::io::Result<()> {
    let mut buf = [0; 1];
    let _ = tcp.read(&mut buf).await?;
    match PackageType::from(buf[0]) {
        PackageType::LobbyConnect => {
            // name length
            let mut buf = [0; 1];
            let _ = tcp.read(&mut buf).await?;
            println!("got name len: {}", buf[0]);
            // name
            let mut buf = vec![0; buf[0].into()];
            let _ = tcp.read(&mut buf).await?;
            let name = String::from_utf8_lossy(&buf).to_string();
            println!("got name buffer: {buf:?}\nas string: {name}");
            println!("{addr} requested a connection; name: {}", name);
            let _ = sender.send(ManagerNotify::Connected { addr: addr.ip(), client: Client::new(name) });
            tcp.writable().await?;
            let client_id = loop {
                match client_event.recv().await.unwrap() {
                    ClientEventBroadcast::Connected {addr: event_addr, client} => {
                        if event_addr == addr.ip() {break client.client_id;} else {continue;}
                    }
                    ClientEventBroadcast::Reconnected {addr: event_addr, client} => {
                        if event_addr == addr.ip() {break client.client_id;} else {continue;}
                    }
                    ClientEventBroadcast::Multiconnect(event_addr) => {
                        if event_addr == addr.ip() {
                            tcp.try_write(&[u8::from(PackageType::LobbyConnectionDeny)])?;
                            return Ok(());
                        } else {continue;}
                    }
                    _ => {}
                }
            };
            let clients = client_list.recv().await.unwrap();
            let response = Vec::from(LobbyConnectionAcceptResponse {
                client_id,
                lobby: Lobby {
                    client_count: clients.len() as u16,
                    game_count: 0,
                    clients,
                    games: HashMap::new(),
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
                    _ => println!("unknown data received... buf: {buf:?}"),
                }
            }
            Ok(event) = client_event.recv() => {
                match event {
                    ClientEventBroadcast::Connected{client, ..} => {
                        LobbyUpdateData::Connect(client).write(&mut tcp).await?;
                    }
                    ClientEventBroadcast::Disconnected(client_id) => {
                        LobbyUpdateData::Disconnect(client_id).write(&mut tcp).await?;
                    }
                    ClientEventBroadcast::ConnectionInterrupt(client_id) => {
                        LobbyUpdateData::ConnectionInterrupt(client_id).write(&mut tcp).await?;
                    }
                    ClientEventBroadcast::Reconnected{client, ..} => {
                        LobbyUpdateData::Reconnect(client.client_id).write(&mut tcp).await?;
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
        println!("converting client to byte stream...\n{client:#?}\n{bytes:?}");
        bytes
    }
}
