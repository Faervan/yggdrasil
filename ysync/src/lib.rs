use std::{io::Write, net::{TcpStream, UdpSocket}};

use bevy_math::{u16, Quat, Vec3};
use strum::{EnumIter, IntoEnumIterator};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub struct ConnectionSocket {
    //id of the game connected to
    game_id: u16,
    //id of the player
    sender_id: u16,
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
}

#[derive(Debug, PartialEq)]
pub struct YPackage {
    //target_game: u16,
    package_type: PackageType,
    sender: u16,
    receiver: Option<u16>,
    data: Option<Data>,
}

#[derive(EnumIter, Clone, Copy, PartialEq, Debug)]
pub enum PackageType {
    Connection,
    Disconnection,
    Message,
    Movement,
    Attack,
}

#[derive(Debug, PartialEq)]
pub enum Data {
    Message(String),
    Movement(Vec3),
    Attack {
        start_pos: Vec3,
        direction: Quat,
        attack_type: YAttackType,
    },
}

#[derive(Debug, PartialEq)]
pub enum YAttackType {
    Bullet,
}

#[derive(EnumIter, Clone, Copy, PartialEq)]
pub enum ReceiverType {
    Host,
    HostAndClient,
    HostAndAllClients,
}

// This struct is used to automatically define the index of the combinations
struct PackageReceiverCombination {
    package_type: PackageType,
    receiver_type: ReceiverType,
}

struct PackageReceiverMap {
    map: Vec<PackageReceiverCombination>,
}

impl PackageReceiverMap {
    fn new() -> PackageReceiverMap {
        let mut map: Vec<PackageReceiverCombination> = vec![];
        for package_type in PackageType::iter() {
            for receiver_type in ReceiverType::iter() {
                map.push(PackageReceiverCombination {package_type, receiver_type});
            }
        }
        PackageReceiverMap {map}
    }
    fn get_by_index(self, index: usize) -> (PackageType, ReceiverType) {
        let combination = &self.map[index];
        return (combination.package_type, combination.receiver_type);
    }
    fn get_index(self, package_type: PackageType, receiver_type: ReceiverType) -> u8 {
        self.map.iter().position(|p| p.package_type == package_type && p.receiver_type == receiver_type).unwrap().try_into().unwrap()
    }
}

impl YPackage {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        let receiver_type = match self.package_type {
            PackageType::Connection | PackageType::Movement | PackageType::Attack => ReceiverType::Host,
            PackageType::Disconnection => ReceiverType::HostAndAllClients,
            PackageType::Message => match self.receiver {
                Some(_) => ReceiverType::HostAndClient,
                None => ReceiverType::HostAndAllClients,
            }
        };
        bytes.push(PackageReceiverMap::new().get_index(self.package_type, receiver_type));
        bytes.extend_from_slice(&self.sender.to_ne_bytes());
        if let Some(receiver) = self.receiver {
            bytes.extend_from_slice(&receiver.to_ne_bytes());
        }
        match &self.data {
            Some(Data::Message(data)) => bytes.extend_from_slice(data.as_bytes()),
            Some(Data::Movement(..)) => {},
            Some(Data::Attack {..}) => {},
            None => {},
        }
        bytes
    }
}

impl From<&[u8]> for YPackage {
    fn from(mut value: &[u8]) -> Self {
        let (package_type, receiver_type) = PackageReceiverMap::new().get_by_index(value[2].into());
        let sender = u16::from_ne_bytes(value[3..5].try_into().unwrap());
        let receiver = match receiver_type {
            ReceiverType::Host | ReceiverType::HostAndAllClients => {
                value = &value[7..];
                None
            },
            ReceiverType::HostAndClient => {
                let r = Some(u16::from_ne_bytes(value[5..7].try_into().unwrap()));
                value = &value[7..];
                r
            }
        };
        let data = match package_type {
            PackageType::Connection | PackageType::Disconnection => None,
            PackageType::Message => Some(Data::Message(String::from_utf8_lossy(value).to_string())),
            PackageType::Movement => None,
            PackageType::Attack => None,
        };
        YPackage {package_type, sender, receiver, data}
    }
}

impl ConnectionSocket {
    pub fn send(mut self, package: YPackage) -> std::io::Result<()> {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(&self.game_id.to_ne_bytes());
        bytes.extend_from_slice(&self.sender_id.to_ne_bytes());
        bytes.extend(&package.as_bytes());
        match package.package_type {
            PackageType::Connection | PackageType::Disconnection | PackageType::Message => {
                self.tcp_stream.write(&bytes)?;
            },
            PackageType::Movement | PackageType::Attack => {
                self.udp_socket.send(&bytes)?;
            },
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ypackage_en_and_decoding_works() {
        println!("\n------\n");
        let pkg = YPackage{
            package_type: PackageType::Message,
            sender: 12788,
            receiver: Some(23900),
            data: Some(Data::Message("hello world".to_string())),
        };
        let mut pkg_as_bytes: Vec<u8> = vec![1,3];
        pkg_as_bytes.extend(pkg.as_bytes());
        println!("pkg_as_bytes: {pkg_as_bytes:?}");
        assert_eq!(pkg, YPackage::from(pkg_as_bytes.as_slice()));
        println!("{pkg:?}");
        println!("\n------\n");
    }
}
