use std::{sync::Arc, time::Duration};

use crossbeam::channel::Sender;
use tokio::{net::UdpSocket, select, sync::mpsc::UnboundedReceiver, time::sleep};

use crate::{UdpFromClient, UdpFromServer, UdpPackage};

pub async fn udp_handler(udp: Arc<UdpSocket>, mut receiver: UnboundedReceiver<UdpPackage>, sender: Sender<UdpFromServer>) {
    tokio::spawn(heartbeat(udp.clone()));
    let mut pkg_index: u16 = 0;
    loop {
        let mut buf = [0; UdpFromServer::MAX_SIZE + 4];
        select! {
            Some(pkg) = receiver.recv() => {
                let _ = udp.send(&UdpFromClient {
                    id: pkg_index,
                    data: pkg
                }.as_bytes()).await;
                pkg_index = pkg_index.wrapping_add(1);
            }
            _ = udp.recv(&mut buf) => {
                match UdpFromServer::from_buf(&buf[4..]) {
                    Ok(pkg) => {
                        let _ = sender.send(pkg);
                    }
                    Err(e) => println!("Got an error while receiving Udp, e: {e}")
                }
            }
        }
    }
}

async fn heartbeat(udp: Arc<UdpSocket>) {
    let mut pkg_index: u16 = 0;
    loop {
        let _ = udp.send(&UdpFromClient {
            id: pkg_index,
            data: UdpPackage::Heartbeat
        }.as_bytes()).await;
        pkg_index = pkg_index.wrapping_add(1);
        sleep(Duration::from_secs(1)).await;
    }
}
