use bevy::prelude::*;

mod components;
mod controll_systems;
mod systems;

use controll_systems::*;
use systems::*;

pub struct GameBasePlugin;

impl Plugin for GameBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                    setup_light,
                    spawn_player,
                    spawn_camera.after(spawn_player),
                    spawn_floor,
                    spawn_enemy,
                ))
            .add_systems(Update, (
                    rotate_player,
                    rotate_camera.before(move_camera),
                    zoom_camera.before(move_camera),
                    move_player,
                    move_camera.after(move_player),
                    respawn_player,
                    player_attack,
                    move_bullets,
                    bullet_hits_attackable,
                ));
    }
}
