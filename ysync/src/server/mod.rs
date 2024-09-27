use std::{collections::{HashMap, VecDeque}, net::{IpAddr, SocketAddr, UdpSocket}, time::Instant};
use tokio::{net::{TcpListener, TcpStream, ToSocketAddrs}, sync::{broadcast::{self, Receiver, Sender}, mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}}};

use crate::{Client, Lobby, LobbyConnectionAcceptResponse, PackageType};

pub struct ClientConnection {
    client: Client,
    last_connection: Instant,
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
        let id = match self.free_ids.pop_front() {
            Some(free_id) => free_id,
            None => self.clients.len() as u16,
        };
        client.client_id = id;
        self.clients[id as usize] = ClientConnection {client, last_connection: Instant::now(), addr};
        self.active_clients.push(id);
        id
    }
    fn remove_client(&mut self, client_id: u16) {
        self.active_clients.retain(|x| *x != client_id);
        self.free_ids.push_back(client_id);
    }
    // This does not check if the client is still active
    fn get_client(&self, client_id: u16) -> &ClientConnection {
        &self.clients[client_id as usize]
    }
    fn get_clients(&self) -> Vec<&ClientConnection> {
        self.active_clients.iter().map(|id| &self.clients[*id as usize]).collect()
    }
}

enum ManagerNotify {
    Connected {
        addr: IpAddr,
        client: Client,
    },
    Disconnected(SocketAddr),
    ConnectionInterrupt(SocketAddr),
}

async fn client_manager(sender: Sender<Vec<&ClientConnection>>, mut receiver: UnboundedReceiver<ManagerNotify>) -> tokio::io::Result<()> {
    let manager = ClientManager::new();
    loop {
        match receiver.recv().await {
            Some(ManagerNotify::Connected { addr, client }) => println!("new client! addr: {addr}\n{client:?} name: {}", client.name),
            _ => println!("shit"),
        }
    }
}

pub async fn listen<A: ToSocketAddrs>(tcp_addr: A) -> std::io::Result<()> {
    // Channel to send data to the client manager
    let (client_send, manager_recv) = unbounded_channel();
    // Channel to receive data from the client manager
    let (manager_send, _) = broadcast::channel(5);
    let listener = TcpListener::bind(tcp_addr).await?;
    tokio::spawn(client_manager(manager_send.clone(), manager_recv));
    loop {
        let (tcp, addr) = listener.accept().await?;
        tokio::spawn(handle_client_tcp(tcp, addr, client_send.clone(), manager_send.subscribe()));
    }
}

async fn handle_client_tcp(
    tcp: TcpStream,
    addr: SocketAddr,
    sender: UnboundedSender<ManagerNotify>,
    client_list: Receiver<Vec<&ClientConnection>>,
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
