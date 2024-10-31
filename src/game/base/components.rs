use bevy::prelude::*;

// This Component indicates that an Entity should be despawned with it's children when leaving
// InGame state
#[derive(Component)]
pub struct GameComponentParent;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainCharacter;

fn no() -> bool {false}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player {
    #[reflect(ignore)]
    #[reflect(default = "no")]
    pub mc: bool,
    pub base_velocity: f32,
    pub name: String,
    pub id: u16,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Npc;

#[derive(Component)]
pub struct NormalCamera;

#[derive(Component)]
pub struct EagleCamera {
    // from player to camera
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
pub struct Walking;

#[derive(Component)]
pub struct GlobalUiPosition {
    pub pos: Vec2,
    pub node_entity: Entity
}
#[derive(Component)]
pub struct Follow {
    pub entity: Entity,
}
