use bevy_math::{Quat, Vec3};
use bevy_transform::components::Transform;
use yserde_bytes::AsBytes;

#[derive(AsBytes, Debug)]
pub enum Udp {
    Data {
        id: u16,
        data: UdpData
    },
    Response(u16)
}

#[derive(AsBytes, Debug, Clone)]
pub enum UdpData {
    FromClient(UdpPackage),
    FromServer {
        sender_id: u16,
        content: UdpPackage
    }
}

impl Default for UdpData {
    fn default() -> Self {
        UdpData::FromClient(UdpPackage::Heartbeat)
    }
}

#[derive(AsBytes, Debug, Default, Clone)]
pub enum UdpPackage {
    Move(YTranslation),
    Attack(YPosition),
    Rotate(YRotation),
    Jump,
    #[default]
    Heartbeat
}

#[derive(AsBytes, Debug, Default, Clone)]
pub struct YTranslation {
    x: f32,
    y: f32,
    z: f32
}

impl From<Vec3> for YTranslation {
    fn from(value: Vec3) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}

impl From<YTranslation> for Vec3 {
    fn from(value: YTranslation) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

#[derive(AsBytes, Debug, Default, Clone)]
pub struct YRotation {
    x: f32,
    y: f32,
    z: f32,
    w: f32
}

impl From<Quat> for YRotation {
    fn from(value: Quat) -> Self {
        let array = value.to_array();
        Self { x: array[0], y: array[1], z: array[2], w: array[3] }
    }
}

impl From<YRotation> for Quat {
    fn from(value: YRotation) -> Self {
        Quat::from_array([value.x, value.y, value.z, value.w])
    }
}

#[derive(AsBytes, Debug, Default, Clone)]
pub struct YPosition {
    translation: YTranslation,
    rotation: YRotation,
}

impl From<Transform> for YPosition {
    fn from(value: Transform) -> Self {
        Self {
            translation: YTranslation::from(value.translation),
            rotation: YRotation::from(value.rotation)
        }
    }
}

impl From<YPosition> for Transform {
    fn from(value: YPosition) -> Self {
        Self {
            translation: Vec3::from(value.translation),
            rotation: Quat::from(value.rotation),
            scale: Vec3::ONE
        }
    }
}
