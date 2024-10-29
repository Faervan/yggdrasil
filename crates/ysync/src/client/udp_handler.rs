use std::{sync::Arc, time::Duration};

use crossbeam::channel::Sender;
use tokio::{net::UdpSocket, select, sync::mpsc::UnboundedReceiver, time::{sleep, sleep_until}};

use crate::{save_udp::SaveUdpSupervisor, UdpFromClient, UdpFromServer, UdpPackage};

pub async fn udp_handler(udp: Arc<UdpSocket>, mut receiver: UnboundedReceiver<UdpPackage>, sender: Sender<(u16, UdpPackage)>) {
    tokio::spawn(heartbeat(udp.clone()));
    let mut supervisor = SaveUdpSupervisor::new();
    loop {
        let mut buf = [0; UdpFromServer::MAX_SIZE + 4];
        select! {
            Some(pkg) = receiver.recv() => {
                let _ = udp.send(&UdpFromClient {
                    id: supervisor.send(pkg.clone()),
                    resend: 0,
                    data: pkg
                }.as_bytes()).await;
            }
            _ = udp.recv(&mut buf) => {
                match UdpFromServer::from_buf(&buf[4..]) {
                    Ok(UdpFromServer::Data { sender_id, data }) => {
                        let _ = sender.send((sender_id, data));
                    }
                    Ok(UdpFromServer::Response(id)) => supervisor.received(id),
                    Err(e) => println!("Got an error while receiving Udp, e: {e}")
                }
            }
            _ = sleep_until(supervisor.next_pkg.1) => {
                let _ = udp.send(&supervisor.resend(supervisor.next_pkg.0).as_bytes()).await;
            }
        }
    }
}

async fn heartbeat(udp: Arc<UdpSocket>) {
    let mut pkg_index: u16 = 0;
    loop {
        let _ = udp.send(&UdpFromClient {
            id: pkg_index,
            resend: 0,
            data: UdpPackage::Heartbeat
        }.as_bytes()).await;
        pkg_index = pkg_index.wrapping_add(1);
        sleep(Duration::from_secs(1)).await;
    }
}
