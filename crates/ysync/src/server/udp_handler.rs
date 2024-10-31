use std::net::{IpAddr, SocketAddr};

use bevy_utils::HashMap;

use tokio::{net::UdpSocket, sync::broadcast::Receiver, time::sleep_until};

use crate::{safe_udp::{SafeUdpSupervisor, UdpRecvMemory}, Udp, UdpData, UdpPackage};

use super::EventBroadcast;

struct AddrManager {
    // Client Ip to (game_id, bool) where bool indicates whether the full SocketAddr has already
    // been inserted
    clients: HashMap<IpAddr, (u16, bool)>,
    // Client id to Ip
    client_ids: HashMap<u16, IpAddr>,
    // Client Ip to id
    client_ips: HashMap<IpAddr, u16>,
    // Last 50 packets received from every client
    memory: HashMap<IpAddr, UdpRecvMemory>,
    // Game id to client Ips + Ports
    games: HashMap<u16, Vec<SocketAddr>>
}

impl AddrManager {
    fn new() -> AddrManager {
        AddrManager {
            clients: HashMap::new(),
            client_ids: HashMap::new(),
            client_ips: HashMap::new(),
            memory: HashMap::new(),
            games: HashMap::new()
        }
    }
    fn game_creation(&mut self, game_id: u16, host_id: u16, host_addr: IpAddr) {
        self.clients.insert(host_addr, (game_id, false));
        self.client_ids.insert(host_id, host_addr);
        self.client_ips.insert(host_addr, host_id);
        self.memory.insert(host_addr, UdpRecvMemory::new());
        self.games.insert(game_id, vec![]);
    }
    fn game_deletion(&mut self, game_id: u16) {
        if let Some(clients) = self.games.remove(&game_id) {
            let _ = clients.iter().map(|c| {
                let ip = c.ip();
                self.clients.remove(&ip);
                self.memory.remove(&ip);
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
        self.memory.insert(client_addr, UdpRecvMemory::new());
    }
    fn game_exit(&mut self, client_id: u16) {
        if let Some(client_addr) = self.client_ids.remove(&client_id) {
            self.client_ips.remove(&client_addr);
            if let Some((game_id, is_registered)) = self.clients.remove(&client_addr) {
                if is_registered {
                    self.games.get_mut(&game_id).map(|clients| clients.retain(|c| c.ip() != client_addr));
                }
            }
            self.memory.remove(&client_addr);
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
    let mut supervisor = SafeUdpSupervisor::new();
    let mut buf = [0; Udp::MAX_SIZE + 4];
    loop {
        tokio::select! {
            // Receive and handle packets from the clients
            Ok((_, sender)) = udp.recv_from(&mut buf) => {
                if let Some(sender_id) = manager.get_client_id(sender) {
                    match Udp::from_buf(&buf[4..]) {
                        Ok(Udp::Data { id, data }) => {
                            // Let the client know we got the pkg
                            let _ = udp.send_to(&Udp::Response(id).as_bytes(), sender).await;
                            // Check if we already got this pkg before
                            if manager.memory.get_mut(&sender.ip()).map(|m| m.check_packet(id)).unwrap_or(false) {
                                if let UdpData::FromClient(content) = data {
                                    match content {
                                        UdpPackage::Heartbeat => {
                                            // Check if the client has already been registered on
                                            // the redirect_list of it's game and register if not
                                            if !manager.is_registered(sender.ip()) {
                                                manager.register_full_addr(sender);
                                            }
                                        }
                                        _ => {
                                            // Forward pkg to all other clients connected to the
                                            // game
                                            let redirect_list = manager.get_redirect_list(sender.ip());
                                            let pkg_data = UdpData::FromServer {
                                                sender_id,
                                                content
                                            };
                                            for client in redirect_list.into_iter() {
                                                if client != sender {
                                                    udp.send_to(&Udp::Data { id, data: pkg_data.clone()}.as_bytes(), client).await?;
                                                    // Remember that we send this pkg, so we can
                                                    // resend if we don't get a response
                                                    supervisor.send(pkg_data.clone());
                                                }
                                            }
                                        }
                                    }
                                } else {println!("unexpectedly got a UdpData::FromServer: {data:?}")}
                            }
                        }
                        Ok(Udp::Response(id)) => supervisor.received(id),
                        Err(e) => println!("Got invalid udp package, e: {e}"),
                    }
                }
            }
            // If we don't get a response in time, resend the package
            _ = sleep_until(supervisor.next_resend.instant) => {
                if let Some(pkg) = supervisor.resend(supervisor.next_resend.id) {
                    let _ = udp.send(&pkg.as_bytes()).await;
                }
            }
            // Get Tcp events and update the AddrManager accordingly
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
