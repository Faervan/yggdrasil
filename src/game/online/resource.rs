use std::time::Duration;

use bevy::prelude::*;

#[derive(Resource)]
pub struct ShareMovementTimer(pub Timer);

#[derive(Resource)]
pub struct ShareRotationTimer(pub Timer);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GameAgeDuration(pub Duration);
