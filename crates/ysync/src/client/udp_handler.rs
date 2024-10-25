use std::{sync::Arc, time::Duration};

use crossbeam::channel::Sender;
use tokio::{net::UdpSocket, select, sync::mpsc::UnboundedReceiver, time::sleep};

use crate::{UdpFromServer, UdpPackage};

pub async fn udp_handler(udp: Arc<UdpSocket>, mut receiver: UnboundedReceiver<UdpPackage>, sender: Sender<UdpFromServer>) {
    tokio::spawn(heartbeat(udp.clone()));
    loop {
        let mut buf = [0; UdpFromServer::MAX_SIZE + 4];
        select! {
            Some(pkg) = receiver.recv() => {
                let _ = udp.send(&pkg.as_bytes()).await;
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
    loop {
        let _ = udp.send(&UdpPackage::Heartbeat.as_bytes()).await;
        sleep(Duration::from_secs(1)).await;
    }
}
