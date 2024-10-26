use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::game::base::components::{MainCharacter, Player};

use super::events::{MovePlayer, PlayerJump, RotatePlayer};

pub fn rotate_other_players(
    mut players: Query<(&mut Transform, &Player), Without<MainCharacter>>,
    mut rotate_events: EventReader<RotatePlayer>,
) {
    for event in rotate_events.read().into_iter() {
        players.iter_mut().find(|(_, p)| p.id == event.id).map(|(mut pos, _)| pos.rotation = event.rotation);
    }
}

pub fn move_other_players(
    mut players: Query<(&mut Transform, &Player), Without<MainCharacter>>,
    mut move_events: EventReader<MovePlayer>,
) {
    for event in move_events.read().into_iter() {
        players.iter_mut().find(|(_, p)| p.id == event.id).map(|(mut pos, _)| pos.translation = event.position);
    }
}

pub fn other_players_jump(
    mut players: Query<(&mut Velocity, &Player), Without<MainCharacter>>,
    mut move_events: EventReader<PlayerJump>,
) {
    for event in move_events.read().into_iter() {
        players.iter_mut().find(|(_, p)| p.id == event.0).map(|(mut velocity, _)| {
            velocity.linvel = Vec3::new(0., 40., 0.);
            velocity.angvel = Vec3::ZERO;
        });
    }
}
