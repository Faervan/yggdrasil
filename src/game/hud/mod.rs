use bevy::prelude::*;
use debug::{build_debug_hud, despawn_debug_hud, try_remove_debug_hud, try_set_debug_hud, update_fps};

use crate::AppState;

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
            .add_systems(Update, update_fps.run_if(in_state(HudDebugState::Enabled)));
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
