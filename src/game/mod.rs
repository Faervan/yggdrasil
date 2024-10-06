use bevy::prelude::*;

pub mod components;
mod controll_systems;
mod systems;

use controll_systems::*;
use systems::*;

use crate::{ui::chat::ChatState, AppState};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<OnlineGame>()
            .add_systems(OnEnter(AppState::InGame), (
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
                    animate_walking,
                ).run_if(in_state(AppState::InGame)))
            .add_systems(OnExit(AppState::InGame), despawn_all_entities);
    }
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum OnlineGame {
    #[default]
    None,
    Host,
    Client,
}

#[derive(Resource)]
pub struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}
