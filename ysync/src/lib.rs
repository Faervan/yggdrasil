use bevy_math::Vec3;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub struct YPackage {
    _package_type: PackageType,
    _sender: YParticipant,
    _receiver: YReceiver,
    _data: Option<Data>,
}

pub enum PackageType {
    Connection,
    Disconnection,
    Message,
    Movement,
    Attack,
}

pub enum YParticipant {
    Client(u16),
    Host,
}

pub enum YReceiver {
    All,
    YParticipant(YParticipant),
}

pub enum Data {
    Message(String),
    Movement(Vec3),
    Attack(YAttack),
}

pub struct YAttack {
    _start_pos: Vec3,
    _direction: Vec3,
    _attack_type: YAttackType,
}

pub enum YAttackType {
    Bullet,
}

impl YPackage {
    pub fn from_buffer(buf: &[u8]) -> YPackage {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
