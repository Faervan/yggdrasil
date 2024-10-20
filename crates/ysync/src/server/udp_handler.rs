use std::net::IpAddr;

use tokio::{net::UdpSocket, sync::mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::{UdpFromServer, UdpPackage};

pub async fn udp_handler(game_request: UnboundedSender<IpAddr>, mut client_list: UnboundedReceiver<(u16, Vec<IpAddr>)>) -> tokio::io::Result<()> {
    let udp = UdpSocket::bind("0.0.0.0:9983").await?;
    let mut buf = [0; UdpPackage::MAX_SIZE + 4];
    loop {
        let (_, sender) = udp.recv_from(&mut buf).await?;
        if let Ok(udp_package) = UdpPackage::from_buf(&buf[4..]) {
            let _ = game_request.send(sender.ip());
            let (sender_id, redirect_list) = client_list.recv().await.expect("Udp to game manager unbounded channel has been closed!");
            let udp_from_server_buf = UdpFromServer { sender_id, data: udp_package }.as_bytes();
            for client in redirect_list.into_iter() {
                if client != sender.ip() {
                    udp.send_to(&udp_from_server_buf, format!("{client}:9983")).await?;
                }
            }
        }
    }
}
