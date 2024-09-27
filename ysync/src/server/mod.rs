use std::{collections::VecDeque, net::{IpAddr, SocketAddr, UdpSocket}, time::Instant};
use tokio::{net::{TcpListener, TcpStream, ToSocketAddrs}, sync::{broadcast::{self, Receiver, Sender}, mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}}};

use crate::{Client, Lobby, LobbyConnectionAcceptResponse, PackageType};

#[derive(Clone)]
pub struct ClientConnection {
    client: Client,
    idle_timeout: Option<Instant>,
    addr: IpAddr,
}

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
    fn add_client(&mut self, mut client: Client, addr: IpAddr) -> u16 {
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
            true => self.clients.push(ClientConnection {client, idle_timeout: None, addr}),
            false => self.clients[id as usize] = ClientConnection {client, idle_timeout: None, addr},
        }
        self.active_clients.push(id);
        id
    }
    fn remove_client(&mut self, addr: IpAddr) {
        let client_id = self.clients.iter().find(|c| c.addr == addr).unwrap().client.client_id;
        self.active_clients.retain(|x| *x != client_id);
        self.free_ids.push_back(client_id);
    }
    // This does not check if the client is still active
    fn get_client(&self, client_id: u16) -> ClientConnection {
        self.clients[client_id as usize].clone()
    }
    fn get_clients(&self) -> Vec<Client> {
        self.active_clients.iter().map(|id| self.clients[*id as usize].client.clone()).collect()
    }
}

enum ManagerNotify {
    Connected {
        addr: IpAddr,
        client: Client,
    },
    Disconnected(IpAddr),
    ConnectionInterrupt(IpAddr),
}

async fn client_manager(
    client_event: Sender<ClientConnection>,
    client_list: Sender<Vec<Client>>,
    mut receiver: UnboundedReceiver<ManagerNotify>
) -> tokio::io::Result<()> {
    let mut manager = ClientManager::new();
    loop {
        match receiver.recv().await {
            Some(ManagerNotify::Connected { addr, client }) => {
                println!("new client! addr: {addr}\n{client:?} name: {}", client.name);
                let client_id = manager.add_client(client, addr);
                let _ = client_event.send(manager.get_client(client_id));
                let _ = client_list.send(manager.get_clients());
            }
            Some(ManagerNotify::Disconnected(addr)) => {
                println!("client disconnected! addr: {addr}");
                manager.remove_client(addr);
            }
            _ => println!("shit"),
        }
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
    tcp: TcpStream,
    addr: SocketAddr,
    sender: UnboundedSender<ManagerNotify>,
    mut client_event: Receiver<ClientConnection>,
    mut client_list: Receiver<Vec<Client>>,
) -> tokio::io::Result<()> {
    /*tcp.writable().await?;
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
    let _ = channel.send(ManagerNotify::Connected { addr: addr.ip(), client: Client {client_id: 7, in_game: false, name} });*/
    let mut buf = [0; 1];
    tcp.readable().await?;
    let _ = tcp.try_read(&mut buf)?;
    match PackageType::from(buf[0]) {
        PackageType::LobbyConnect => {
            let mut buf = [0; 25];
            tcp.readable().await?;
            let _ = tcp.try_read(&mut buf)?;
            println!("{addr} requested a connection; name: {}", String::from_utf8_lossy(&buf));
            let _ = sender.send(ManagerNotify::Connected { addr: addr.ip(), client: Client::new(String::from_utf8_lossy(&buf).to_string()) });
            tcp.writable().await?;
            let client_id = client_event.recv().await.unwrap().client.client_id;
            let clients = client_list.recv().await.unwrap();
            let response = Vec::from(LobbyConnectionAcceptResponse {
                client_id,
                lobby: Lobby {
                    client_count: clients.len() as u16,
                    game_count: 0,
                    clients,
                    games: vec![],
                }
            });
            tcp.try_write(response.as_slice())?;
        }
        PackageType::LobbyDisconnect => {
            println!("{addr} requested a disconnect");
        }
        _ => {}
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
        for client in response.lobby.clients {
            //client_id
            bytes.extend_from_slice(&client.client_id.to_ne_bytes());
            //in_game
            bytes.push(0);
            //name_len
            bytes.push(client.name.len() as u8);
            //name
            bytes.extend_from_slice(client.name.as_bytes());
        }
        bytes
    }
}
