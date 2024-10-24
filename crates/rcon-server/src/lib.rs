use std::io::{Error, ErrorKind};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::{mpsc::UnboundedSender, oneshot::{self, Sender}}, task};

pub async fn listen(port: Option<u16>, password: impl ToString, channel: UnboundedSender<(Sender<String>, String)>) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port.unwrap_or(27015))).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        task::spawn(handle_connection(stream, password.to_string(), channel.clone()));
    }
}

async fn handle_connection(mut stream: TcpStream, password: String, channel: UnboundedSender<(Sender<String>, String)>) -> tokio::io::Result<()> {
    let mut buf = [0; 4100];
    let mut authenticated = false;
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            return Err(Error::new(ErrorKind::ConnectionAborted, "Got 0 byte length while reading tcp"));
        }
        if let Ok(packet) = Packet::try_from(&buf[..n]) {
            match packet.packet_type {
                PacketType::ServerdataAuth => {
                    stream.write(&packet.empty_response_value()).await?;
                    let id = match packet.body == password {
                        true => {
                            authenticated = true;
                            packet.id
                        }
                        false => -1
                    };
                    stream.write(&Vec::from(Packet {id, packet_type: PacketType::ServerdataAuthResponse, body: "".to_string()})).await?;
                }
                PacketType::ServerdataExeccommand => {
                    if authenticated {
                        let (sx, rx) = oneshot::channel();
                        let _ = channel.send((sx, packet.body));
                        let response_body = match rx.await {
                            Ok(value) => value,
                            Err(e) => return Err(Error::new(ErrorKind::BrokenPipe, e))
                        };
                        stream.write(&Vec::from(Packet {
                            id: packet.id,
                            packet_type: PacketType::ServerdataResponseValue,
                            body: response_body
                        })).await?;
                    }
                }
                PacketType::ServerdataResponseValue => {
                    let pkg_as_bytes = Vec::from(packet);
                    stream.write(&pkg_as_bytes).await?;
                    stream.write(&pkg_as_bytes).await?;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
struct Packet {
    id: i32,
    packet_type: PacketType,
    body: String,
}

impl Packet {
    fn empty_response_value(&self) -> Vec<u8> {
        Vec::from(Packet {
            id: self.id,
            packet_type: PacketType::ServerdataResponseValue,
            body: "".to_string()
        })
    }
}

#[derive(Debug)]
enum PacketType {
    ServerdataAuth,
    ServerdataAuthResponse,
    ServerdataExeccommand,
    ServerdataResponseValue,
}

impl TryFrom<&[u8]> for Packet {
    type Error = &'static str;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let size = i32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
        let id = i32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let packet_type = match i32::from_le_bytes(bytes[8..12].try_into().unwrap()) {
            3 => PacketType::ServerdataAuth,
            2 => PacketType::ServerdataExeccommand,
            0 => PacketType::ServerdataResponseValue,
            _ => return Err("Invalid PacketType")
        };
        let body = String::from_utf8_lossy(&bytes[12..size+2]).to_string();
        Ok(Packet {
            id,
            packet_type,
            body
        })
    }
}

impl From<Packet> for Vec<u8> {
    fn from(packet: Packet) -> Self {
        let mut bytes = vec![];
        bytes.extend_from_slice(&packet.id.to_le_bytes());
        bytes.extend_from_slice(&{
            (match packet.packet_type {
                PacketType::ServerdataAuth => 3,
                PacketType::ServerdataAuthResponse => 2,
                PacketType::ServerdataExeccommand => 2,
                PacketType::ServerdataResponseValue => 0
            }) as i32
        }.to_le_bytes());
        let body: Vec<u8> = packet.body.as_bytes().into_iter().filter(|b| **b != 0).cloned().collect();
        bytes.extend_from_slice(&body);
        bytes.extend_from_slice(&[0; 2]);
        let mut sized_bytes = vec![];
        sized_bytes.extend((bytes.len() as i32).to_le_bytes());
        sized_bytes.extend(bytes);
        sized_bytes
    }
}
