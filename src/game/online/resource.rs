use bevy::prelude::*;

#[derive(Resource)]
pub struct ShareMovementTimer(pub Timer);

#[derive(Resource)]
pub struct ShareRotationTimer(pub Timer);
