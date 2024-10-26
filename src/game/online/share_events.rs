use bevy::prelude::*;
use ysync::{UdpPackage, YPosition, YRotation, YTranslation};

use crate::ui::lobby::LobbySocket;

use super::{events::{ShareAttack, ShareMovement, ShareRotation}, resource::{ShareMovementTimer, ShareRotationTimer}};

pub fn advance_timers(
    mut share_movement: ResMut<ShareMovementTimer>,
    mut share_rotation: ResMut<ShareRotationTimer>,
    time: Res<Time>,
) {
    share_movement.0.tick(time.delta());
    share_rotation.0.tick(time.delta());
}

pub fn share_movement(
    remote: Res<LobbySocket>,
    mut movement_event: EventReader<ShareMovement>,
) {
    let event = movement_event.read().next().expect("All according to plan of course");
    let _ = remote.socket.udp_send.send(UdpPackage::Move(YTranslation::from(event.0)));
}

pub fn share_rotation(
    socket: Res<LobbySocket>,
    mut rotation_event: EventReader<ShareRotation>,
) {
    let event = rotation_event.read().next().expect("All according to plan of course");
    let _ = socket.socket.udp_send.send(UdpPackage::Rotate(YRotation::from(event.0)));
}

pub fn share_jump(
    remote: Res<LobbySocket>,
) {
    let _ = remote.socket.udp_send.send(UdpPackage::Jump);
}

pub fn share_attack(
    socket: Res<LobbySocket>,
    mut attack_event: EventReader<ShareAttack>,
) {
    let event = attack_event.read().next().expect("All according to plan of course");
    let _ = socket.socket.udp_send.send(UdpPackage::Attack(YPosition::from(event.0)));
}
