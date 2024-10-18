use std::{collections::VecDeque, net::IpAddr};

use bevy_utils::HashMap;
use tokio::time::Instant;

use crate::{Client, ClientStatus};

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
            false => ClientStatus::Idle(self.last_con.elapsed().as_secs() as u16),
        };
        client
    }
}

#[derive(Debug)]
pub struct ClientManager {
    clients: Vec<ClientConnection>,
    active_clients: Vec<u16>,
    free_ids: VecDeque<u16>,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        ClientManager {
            clients: Vec::new(),
            active_clients: Vec::new(),
            free_ids: VecDeque::new(),
        }
    }
    pub fn add_client(&mut self, client: &mut Client, addr: IpAddr) -> Option<bool> {
        if let Some(client) = self.clients.iter_mut().find(|c| c.addr == addr) {
            if let Some(_) = self.active_clients.iter().find(|a| **a == client.client.client_id) {
                match client.active {
                    true => return None,
                    false => {
                        client.active = true;
                        return Some(true);
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
            true => self.clients.push(ClientConnection {client: client.clone(), active: true, last_con: Instant::now(), addr}),
            false => self.clients[id as usize] = ClientConnection {client: client.clone(), active: true, last_con: Instant::now(), addr},
        }
        self.active_clients.push(id);
        Some(false)
    }
    pub fn remove_client(&mut self, addr: IpAddr) -> u16 {
        let client_id = self.clients.iter().find(|c| c.addr == addr).unwrap().client.client_id;
        self.active_clients.retain(|a| *a != client_id);
        self.free_ids.push_back(client_id);
        client_id
    }
    pub fn get_client(&self, client_id: u16) -> Client {
        self.clients[client_id as usize].as_client()
    }
    pub fn get_clients(&self) -> HashMap<u16, Client> {
        self.active_clients.iter().map(|id| (*id, self.clients[*id as usize].client.clone())).collect()
    }
    pub fn inactivate_client(&mut self, addr: IpAddr) -> u16 {
        let client = self.clients.iter_mut().find(|c| c.addr == addr).unwrap();
        client.active = false;
        client.last_con = Instant::now();
        client.client.client_id
    }
}
