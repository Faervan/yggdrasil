use bevy::prelude::*;

pub mod components;
mod controll_systems;
mod systems;

use controll_systems::*;
use systems::*;

use crate::{ui::chat::ChatState, AppState, GameSessionType};

pub struct GameBasePlugin;

impl Plugin for GameBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame(GameSessionType::Singleplayer)), (
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
                    move_player.run_if(not(in_state(ChatState::Open))),
                    move_camera.after(move_player),
                    respawn_player,
                    player_attack,
                    move_bullets,
                    bullet_hits_attackable,
                    return_to_menu.run_if(not(in_state(ChatState::Open))),
                ).run_if(in_state(AppState::InGame(GameSessionType::Singleplayer))))
            .add_systems(OnExit(AppState::InGame(GameSessionType::Singleplayer)), despawn_all_entities);
    }
}
