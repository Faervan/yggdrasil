use std::{collections::VecDeque, time::Duration, u16};

use bevy_utils::HashMap;
use tokio::time::Instant;

use crate::{UdpFromClient, UdpPackage};

#[derive(Clone)]
pub struct SaveUdpSupervisor {
    index: u16,
    ping_stats: VecDeque<Duration>,
    pub ping: Duration,
    pub next_pkg: (u16, Instant),
    packets: HashMap<u16, PacketStat>,
}

impl SaveUdpSupervisor {
    pub fn new() -> Self {
        SaveUdpSupervisor {
            index: 0,
            ping_stats: VecDeque::new(),
            ping: Duration::from_millis(200),
            next_pkg: (u16::MAX, Instant::now() + Duration::from_secs(300)),
            packets: HashMap::new()
        }
    }
    pub fn send(&mut self, pkg: UdpPackage) -> u16 {
        let id = self.index;
        self.index = self.index.wrapping_add(1);
        let now = Instant::now();
        let response_time = now + self.ping;
        self.packets.insert(id, PacketStat {
            time: now,
            expected_response: response_time,
            resend: 0,
            data: pkg
        });
        if response_time < self.next_pkg.1 {
            self.next_pkg.1 = response_time;
        }
        self.next_pkg = (id, response_time);
        id
    }
    pub fn received(&mut self, id: u16) {
        if let Some(pkg) = self.packets.remove(&id) {
            self.ping_stats.push_back(pkg.time.elapsed());
            if self.ping_stats.len() > 10 {
                self.ping_stats.pop_front();
            }
            self.ping = (self.ping_stats.iter().sum::<Duration>() / self.ping_stats.len().try_into().unwrap()) * 12 / 10;
            if id == self.next_pkg.0 {
                self.next_pkg = self.packets.
                    iter()
                    .min_by(|x, y| x.1.expected_response.cmp(&y.1.expected_response))
                    .map(|(id, pkg)| (*id, pkg.expected_response))
                    .unwrap_or((u16::MAX, Instant::now() + Duration::from_secs(300)));
            }
        }
    }
    pub fn resend(&mut self, id: u16) -> UdpFromClient {
        let pkg = self.packets.get_mut(&id).unwrap();
        pkg.resend += 1;
        UdpFromClient {
            id,
            resend: pkg.resend,
            data: pkg.data.clone()
        }
        
    }
}

#[derive(Clone)]
struct PacketStat {
    time: Instant,
    expected_response: Instant,
    resend: u8,
    data: UdpPackage
}
