use bevy::prelude::*;

use super::resources::Animations;

// This Component indicates that an Entity should be despawned with it's children when leaving
// InGame state
#[derive(Component)]
pub struct GameComponentParent;
#[derive(Component)]
pub struct GameComponent;

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

#[derive(Component, Debug)]
pub enum AnimationState {
    Idle,
    Walk(WalkDirection),
    Sprint(WalkDirection),
}

#[derive(Component, Debug)]
pub enum WalkDirection {
    Forward,
    Back,
    Left,
    Right,
    ForwardLeft,
    ForwardRight,
    BackLeft,
    BackRight
}

impl WalkDirection {
    fn from_keys(w: bool, a: bool, s: bool, d: bool) -> Option<Self> {
        if w && !s {
            return Some(match (a, d) {
                (true, false) => WalkDirection::ForwardLeft,
                (false, true) => WalkDirection::ForwardRight,
                _ => WalkDirection::Forward
            });
        } else if s && !w {
            return Some(match (a, d) {
                (true, false) => WalkDirection::BackLeft,
                (false, true) => WalkDirection::BackRight,
                _ => WalkDirection::Back
            });
        } else if a || d && (!a || !d) { // only one of them
            return Some(match a {
                true => WalkDirection::Left,
                false => WalkDirection::Right,
            });
        }
        None
    }
}

impl AnimationState {
    pub fn get_node(&self, ani: &Animations) -> AnimationNodeIndex {
        ani.animations[match self {
            AnimationState::Idle => 0,
            AnimationState::Walk(_) => 2,
            AnimationState::Sprint(_) => 1,
        }]
    }
    pub fn from_input(input: &ButtonInput<KeyCode>) -> Option<Self> {
        let keys = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::ShiftLeft];
        let just_pressed = keys.iter().map(|k| input.just_pressed(*k)).collect::<Vec<bool>>();
        let released = keys.iter().map(|k| input.just_released(*k)).collect::<Vec<bool>>();
        let pressed = keys.iter().map(|k| input.pressed(*k)).collect::<Vec<bool>>();
        if let Some(_) = just_pressed
            .iter()
            .chain(released.iter())
            .find(|b| **b == true)
        {
            return Some(WalkDirection::from_keys(pressed[0], pressed[1], pressed[2], pressed[3])
                .map(|dir| match input.pressed(KeyCode::ShiftLeft) {
                    true => AnimationState::Sprint(dir),
                    false => AnimationState::Walk(dir),
                })
                .unwrap_or(AnimationState::Idle));
        }
        None
    }
}

#[derive(Component)]
pub struct GlobalUiPosition {
    pub pos: Vec2,
    pub node_entity: Entity
}
#[derive(Component)]
pub struct Follow {
    pub entity: Entity,
}
