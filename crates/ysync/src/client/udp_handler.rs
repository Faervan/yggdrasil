use crossbeam::channel::Sender;
use tokio::{net::UdpSocket, select, sync::mpsc::UnboundedReceiver};

use crate::{UdpFromServer, UdpPackage};

pub async fn udp_handler(udp: UdpSocket ,mut receiver: UnboundedReceiver<UdpPackage>, sender: Sender<UdpFromServer>) {
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
