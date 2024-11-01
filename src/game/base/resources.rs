use std::time::{Duration, Instant};

use bevy::prelude::*;

#[derive(Resource)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}

#[derive(Resource)]
pub struct PlayerName(pub String);

#[derive(Resource)]
pub struct PlayerId(pub u16);

#[derive(Resource)]
pub struct TimeInGame(pub Time<Real>);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GameAge {
    pub startup: Instant,
    pub time: Time<Real>
}

impl Default for GameAge {
    fn default() -> Self {
        let instant = Instant::now();
        GameAge {
            startup: instant,
            time: Time::new(instant)
        }
    }
}

impl GameAge {
    pub fn from_duration(age: Duration) -> Self {
        let time = Instant::now() - age;
        GameAge {
            startup: time,
            time: Time::new(time)
        }
    }
}

#[derive(Resource)]
pub struct WorldScene(pub Handle<DynamicScene>);

#[derive(Resource)]
pub struct CameraDirection(pub Vec2);
