use bevy::prelude::*;

#[derive(Component)]
pub struct GameComponentParent;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainCharacter;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player {
    pub base_velocity: f32,
    pub name: String,
    pub id: u16,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Npc;

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
    pub shooter: u16,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health {
    pub value: u32,
}

#[derive(Component)]
pub struct IsWalking;
