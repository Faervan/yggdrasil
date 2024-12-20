use animations::{animate, setup_animation};
use bevy::{animation::animate_targets, prelude::*};
use bevy_atmosphere::plugin::AtmospherePlugin;
use camera::CameraPlugin;
use cursor::CursorPlugin;
use light::setup_light;
use misc_systems::{advance_time, compute_screen_positions, despawn_all_entities, follow_for_node, insert_game_age, insert_in_game_time, return_to_menu, toggle_debug};
use npcs::{insert_npc_components, spawn_npc};
use players::PlayerPlugin;
use projectiles::{bullet_hits_attackable, move_bullets};
use resources::{GameAge, PlayerId, PlayerName};
use scene_setup::spawn_floor;

use crate::{ui::chat::ChatState, AppState};

use super::online::OnlineState;

pub mod components;
pub mod resources;
mod animations;
mod camera;
mod cursor;
mod light;
mod misc_systems;
mod npcs;
pub mod projectiles;
mod scene_setup;
pub mod players;

pub struct GameBasePlugin;

impl Plugin for GameBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                CameraPlugin,
                AtmospherePlugin,
                PlayerPlugin,
                CursorPlugin
            ))
            .insert_resource(PlayerName("Jon".to_string()))
            .insert_resource(PlayerId(0))
            .add_systems(OnEnter(AppState::InGame), (
                setup_light,
                spawn_floor,
                spawn_npc.run_if(not(in_state(OnlineState::Client))),
                insert_in_game_time,
                insert_game_age.run_if(not(in_state(OnlineState::Client))),
            ))
            .add_systems(Update, (
                advance_time.run_if(resource_exists::<GameAge>),
                move_bullets,
                bullet_hits_attackable,
                setup_animation.before(animate_targets),
                animate.before(animate_targets),
                toggle_debug,
                insert_npc_components,
                compute_screen_positions,
                follow_for_node,
            ).run_if(in_state(AppState::InGame)))
            .add_systems(Update, return_to_menu
                .run_if(not(in_state(ChatState::Open))).run_if(in_state(AppState::InGame)).run_if(in_state(OnlineState::None)))
            .add_systems(OnExit(AppState::InGame), despawn_all_entities);
    }
}
