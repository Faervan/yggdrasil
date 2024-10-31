use bevy::prelude::*;
use debug::{build_debug_hud, despawn_debug_hud, try_remove_debug_hud, try_set_debug_hud, update_fps, update_game_age, update_in_game_time, update_ping};

use crate::AppState;

use super::base::resources::{GameAge, TimeInGame};

mod debug;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<HudDebugState>()
            .insert_resource(HudParentEntities {
                debug: None,
            })
            .add_systems(OnEnter(AppState::InGame), try_set_debug_hud)
            .add_systems(OnExit(AppState::InGame), try_remove_debug_hud)
            .add_systems(OnEnter(HudDebugState::Enabled), build_debug_hud)
            .add_systems(OnExit(HudDebugState::Enabled), despawn_debug_hud)
            .add_systems(Update, (
                update_fps,
                update_ping,
                update_in_game_time.run_if(resource_exists::<TimeInGame>),
                update_game_age.run_if(resource_exists::<GameAge>),
            ).run_if(in_state(HudDebugState::Enabled)));
    }
}

#[derive(States, Default, Hash, PartialEq, Eq, Clone, Debug)]
pub enum HudDebugState {
    Enabled,
    #[default]
    Disabled,
}

#[derive(Resource)]
struct HudParentEntities {
    debug: Option<Entity>
}

#[derive(Resource)]
struct FpsInfo {
    timer: Timer,
    last_fps: u32,
}

#[derive(Component)]
struct FpsInfoText;

#[derive(Component)]
struct InGameTimeInfoText;

#[derive(Component)]
struct GameAgeInfoText;

#[derive(Component)]
struct PingInfoText;
