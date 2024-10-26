use bevy::prelude::*;

#[derive(Event)]
pub struct ShareWorld;

#[derive(Event)]
pub struct ReceivedWorld(pub String);

#[derive(Event)]
pub struct ShareRotation(pub Quat);

#[derive(Event)]
pub struct ShareMovement(pub Vec3);

#[derive(Event)]
pub struct ShareJump;

#[derive(Event)]
pub struct ShareAttack(pub Transform);

#[derive(Event)]
pub struct MovePlayer {
    pub id: u16,
    pub position: Vec3
}

#[derive(Event)]
pub struct RotatePlayer {
    pub id: u16,
    pub rotation: Quat
}

#[derive(Event)]
pub struct PlayerJump(pub u16);

#[derive(Event)]
pub struct PlayerAttack {
    pub player_id: u16,
    pub position: Transform
}

#[derive(Event)]
pub struct SpawnPlayer {
    pub name: String,
    pub id: u16,
    pub position: Transform
}

#[derive(Event)]
pub struct DespawnPlayer(pub u16);
