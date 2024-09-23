use std::{collections::HashMap, net::{TcpStream, UdpSocket}};

use bevy_math::{u16, Quat, Vec3};
use strum::{EnumIter, IntoEnumIterator};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub struct TargetGame {
    //id of the game connected to
    game_id: u16,
    //id of the player
    id: u16,
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
}

#[derive(Debug)]
pub struct YPackage {
    target_game: u16,
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

#[derive(Debug)]
pub enum Data {
    Message(String),
    Movement(Vec3),
    Attack {
        start_pos: Vec3,
        direction: Quat,
        attack_type: YAttackType,
    },
}

#[derive(Debug)]
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
    fn get_index(self, package_type: PackageType, receiver_type: ReceiverType) -> usize {
        self.map.iter().position(|p| p.package_type == package_type && p.receiver_type == receiver_type).unwrap()
    }
}

impl From<&[u8]> for YPackage {
    fn from(mut value: &[u8]) -> Self {
        let target_game = u16::from_ne_bytes(value[..2].try_into().unwrap());
        let (package_type, receiver_type) = PackageReceiverMap::new().get_by_index(value[2].into());
        let sender = u16::from_ne_bytes(value[3..5].try_into().unwrap());
        let receiver = match receiver_type {
            ReceiverType::Host | ReceiverType::HostAndAllClients => {
                value = &value[5..];
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
        YPackage {target_game, package_type, sender, receiver, data}
    }
}

impl TargetGame {
    //pub fn from_buffer(buf: &[u8]) -> YPackage {}
    pub fn send_package(self, package_type: PackageType, receiver: Option<u16>, data: Option<Data>) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        println!("\n------\n");
        let vector: Vec<u8> = vec![1, 2, 7, 4, 3, 5, 2, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100];
        let buf: &[u8] = vector.as_slice();
        let package = YPackage::from(buf);
        println!("{package:?}");
    }
}
