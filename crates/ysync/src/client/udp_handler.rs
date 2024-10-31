use std::time::Duration;

use crossbeam::channel::Sender;
use tokio::{net::UdpSocket, select, sync::{mpsc::UnboundedReceiver, watch}, time::{sleep_until, Instant}};

use crate::{safe_udp::{SafeUdpSupervisor, UdpRecvMemory}, Udp, UdpData, UdpPackage};

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(1);

pub async fn udp_handler(udp: UdpSocket, mut receiver: UnboundedReceiver<UdpPackage>, sender: Sender<(u16, UdpPackage)>, ping: watch::Sender<Duration>) {
    let mut supervisor = SafeUdpSupervisor::new();
    let mut recv_memory = UdpRecvMemory::new();
    let mut next_heartbeat = Instant::now() + HEARTBEAT_TIMEOUT;
    let mut buf = [0; Udp::MAX_SIZE + 4];
    loop {
        select! {
            Some(pkg) = receiver.recv() => {
                let _ = udp.send(&Udp::Data {
                    id: supervisor.send(UdpData::FromClient(pkg.clone())),
                    data: UdpData::FromClient(pkg)
                }.as_bytes()).await;
            }
            _ = sleep_until(next_heartbeat) => {
                let _ = udp.send(&Udp::Data {
                    id: supervisor.send(UdpData::FromClient(UdpPackage::Heartbeat)),
                    data: UdpData::FromClient(UdpPackage::Heartbeat)
                }.as_bytes()).await;
                next_heartbeat = Instant::now() + HEARTBEAT_TIMEOUT;
            }
            Ok(n) = udp.recv(&mut buf) => {
                match Udp::from_buf(&buf[4..n]) {
                    Ok(Udp::Data { id,  data }) => {
                        if let UdpData::FromServer { sender_id, content } = data {
                            if recv_memory.check_packet(id) {
                                let _ = sender.send((sender_id, content));
                            }
                            let _ = udp.send(&Udp::Response(id).as_bytes()).await;
                        }
                    }
                    Ok(Udp::Response(id)) => {
                        let _ = ping.send(supervisor.ping);
                        supervisor.received(id)
                    },
                    Err(e) => println!("Got an error while receiving Udp, e: {e}")
                }
            }
            _ = sleep_until(supervisor.next_resend.instant) => {
                if let Some(pkg) = supervisor.resend(supervisor.next_resend.id) {
                    let _ = udp.send(&pkg.as_bytes()).await;
                }
            }
        }
    }
}
