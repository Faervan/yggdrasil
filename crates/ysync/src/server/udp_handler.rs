use std::net::{IpAddr, SocketAddr};

use bevy_utils::HashMap;

use tokio::{net::UdpSocket, sync::broadcast::Receiver};

use crate::{UdpFromServer, UdpPackage};

use super::EventBroadcast;

struct AddrManager {
    // Client Ip to (game_id, bool) where bool indicates whether the full SocketAddr has already
    // been inserted
    clients: HashMap<IpAddr, (u16, bool)>,
    // Client id to Ip
    client_ids: HashMap<u16, IpAddr>,
    // Client Ip to id
    client_ips: HashMap<IpAddr, u16>,
    // Game id to client Ips + Ports
    games: HashMap<u16, Vec<SocketAddr>>
}

impl AddrManager {
    fn new() -> AddrManager {
        AddrManager {
            clients: HashMap::new(),
            client_ids: HashMap::new(),
            client_ips: HashMap::new(),
            games: HashMap::new()
        }
    }
    fn game_creation(&mut self, game_id: u16, host_id: u16, host_addr: IpAddr) {
        self.clients.insert(host_addr, (game_id, false));
        self.client_ids.insert(host_id, host_addr);
        self.client_ips.insert(host_addr, host_id);
        self.games.insert(game_id, vec![]);
    }
    fn game_deletion(&mut self, game_id: u16) {
        if let Some(clients) = self.games.remove(&game_id) {
            let _ = clients.iter().map(|c| {
                self.clients.remove(&c.ip());
                if let Some(id) = self.client_ips.remove(&c.ip()) {
                    self.client_ids.remove(&id);
                }
            });
        }
    }
    fn game_entry(&mut self, client_id: u16, client_addr: IpAddr, game_id: u16) {
        self.clients.insert(client_addr, (game_id, false));
        self.client_ids.insert(client_id, client_addr);
        self.client_ips.insert(client_addr, client_id);
    }
    fn game_exit(&mut self, client_id: u16) {
        if let Some(client_addr) = self.client_ids.remove(&client_id) {
            self.client_ips.remove(&client_addr);
            if let Some((game_id, is_registered)) = self.clients.remove(&client_addr) {
                if is_registered {
                    self.games.get_mut(&game_id).map(|clients| clients.retain(|c| c.ip() != client_addr));
                }
            }
        }
    }
    fn get_client_id(&self, client_addr: SocketAddr) -> Option<u16> {
        self.client_ips.get(&client_addr.ip()).copied()
    }
    fn get_redirect_list(&self, client_addr: IpAddr) -> Vec<SocketAddr> {
        let default = vec![];
        self.clients.get(&client_addr).map(|(game_id, _)| self.games.get(game_id).unwrap_or(&default)).unwrap_or(&default).to_vec()
    }
    fn is_registered(&self, client_addr: IpAddr) -> bool {
        self.clients.get(&client_addr).unwrap().1
    }
    fn register_full_addr(&mut self, client_addr: SocketAddr) {
        if let Some((game_id, is_registered)) = self.clients.get_mut(&client_addr.ip()) {
            *is_registered = true;
            self.games.get_mut(game_id).map(|clients| clients.push(client_addr));
        }
    }
}

pub async fn udp_handler(mut event_broadcast: Receiver<EventBroadcast>) -> tokio::io::Result<()> {
    let udp = UdpSocket::bind("0.0.0.0:9983").await?;
    let mut manager = AddrManager::new();
    let mut buf = [0; UdpPackage::MAX_SIZE + 4];
    loop {
        tokio::select! {
            Ok((_, sender)) = udp.recv_from(&mut buf) => {
                if let Ok(udp_package) = UdpPackage::from_buf(&buf[4..]) {
                    if let Some(sender_id) = manager.get_client_id(sender) {
                        match udp_package {
                            UdpPackage::Heartbeat => {
                                if !manager.is_registered(sender.ip()) {
                                    manager.register_full_addr(sender);
                                }
                            }
                            _ => {
                                let redirect_list = manager.get_redirect_list(sender.ip());
                                let udp_from_server_buf = UdpFromServer { sender_id, data: udp_package }.as_bytes();
                                for client in redirect_list.into_iter() {
                                    if client != sender {
                                        udp.send_to(&udp_from_server_buf, client).await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(event) = event_broadcast.recv() => {
                match event {
                    EventBroadcast::GameCreation { game, host_addr } => {
                        manager.game_creation(game.game_id, game.host_id, host_addr);
                    }
                    EventBroadcast::GameDeletion(game_id) => {
                        manager.game_deletion(game_id);
                    }
                    EventBroadcast::GameEntry { client_id, client_addr, game_id } => {
                        manager.game_entry(client_id, client_addr, game_id);
                    }
                    EventBroadcast::GameExit(client_id) => {
                        manager.game_exit(client_id);
                    }
                    _ => {}
                }
            }
        }
    }
}
