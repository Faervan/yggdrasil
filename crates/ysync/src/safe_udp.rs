use std::{collections::{HashSet, VecDeque}, time::Duration, u16};

use bevy_utils::HashMap;
use tokio::time::Instant;

use crate::{Udp, UdpData};

#[derive(Clone)]
pub struct SafeUdpSupervisor {
    index: u16,
    ping_stats: VecDeque<Duration>,
    pub ping: Duration,
    pub next_resend: NextPkg,
    packets: HashMap<u16, PacketStat>,
}

#[derive(Clone)]
pub struct NextPkg {
    pub id: u16,
    pub instant: Instant
}

impl Default for NextPkg {
    fn default() -> Self {
        NextPkg {
            id: u16::MAX,
            instant: Instant::now() + Duration::from_secs(300)
        }
    }
}

impl SafeUdpSupervisor {
    pub fn new() -> Self {
        SafeUdpSupervisor {
            index: 0,
            ping_stats: VecDeque::new(),
            ping: Duration::from_millis(200),
            next_resend: NextPkg::default(),
            packets: HashMap::new()
        }
    }
    pub fn send(&mut self, pkg: UdpData) -> u16 {
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
        if response_time < self.next_resend.instant {
            self.next_resend.instant = response_time;
        }
        self.next_resend = NextPkg { id, instant: response_time };
        id
    }
    pub fn received(&mut self, id: u16) {
        if let Some(pkg) = self.packets.remove(&id) {
            self.ping_stats.push_back(pkg.time.elapsed());
            if self.ping_stats.len() > 10 {
                self.ping_stats.pop_front();
            }
            self.ping = (self.ping_stats.iter().sum::<Duration>() / self.ping_stats.len().try_into().unwrap()) * 12 / 10;
            if id == self.next_resend.id {
                self.next_resend = self.packets.
                    iter()
                    .min_by(|x, y| x.1.expected_response.cmp(&y.1.expected_response))
                    .map(|(id, pkg)| NextPkg {
                        id: *id,
                        instant: pkg.expected_response
                    })
                    .unwrap_or(NextPkg::default());
            }
        }
    }
    pub fn resend(&mut self, id: u16) -> Udp {
        let pkg = self.packets.get_mut(&id).unwrap();
        pkg.resend += 1;
        Udp::Data {
            id,
            data: pkg.data.clone()
        }
        
    }
}

#[derive(Clone)]
struct PacketStat {
    time: Instant,
    expected_response: Instant,
    resend: u8,
    data: UdpData
}

pub struct UdpRecvMemory {
    unique: HashSet<u16>,
    order: VecDeque<u16>
}

impl UdpRecvMemory {
    pub fn new() -> Self {
        UdpRecvMemory {
            unique: HashSet::new(),
            order: VecDeque::new()
        }
    }
    pub fn check_packet(&mut self, id: u16) -> bool {
        let is_new = self.unique.insert(id);
        if is_new {
            self.order.push_back(id);
            if self.order.len() > 50 {
                self.order.pop_front();
            }
        }
        is_new
    }
}
