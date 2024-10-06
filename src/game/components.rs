use bevy::prelude::*;

#[derive(Component)]
pub struct GameComponentParent;

#[derive(Component)]
pub struct Player {
    pub base_velocity: f32,
}

#[derive(Component)]
pub struct Camera {
    pub direction: Vec3,
    pub distance: f32,
}

#[derive(Component)]
pub struct Bullet {
    pub origin: Vec3,
    pub range: f32,
    pub velocity: f32,
}

#[derive(Component)]
pub struct Attackable;

#[derive(Component)]
pub struct Health {
    pub value: u32,
}

#[derive(Component)]
pub struct IsWalking;
