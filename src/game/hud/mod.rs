use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, DefaultInspectorConfigPlugin};
use debug::*;

use crate::{ui::lobby::ConnectionState, AppState};

use super::online::OnlineState;

mod debug;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<HudDebugState>()
            .add_plugins((
                EguiPlugin,
                DefaultInspectorConfigPlugin,
            ))
            .insert_resource(HudParentEntities {
                debug: None,
            })
            .add_systems(OnEnter(AppState::InGame), try_set_debug_hud)
            .add_systems(OnExit(AppState::InGame), try_remove_debug_hud)
            .add_systems(OnEnter(HudDebugState::Enabled), build_debug_hud)
            .add_systems(OnExit(HudDebugState::Enabled), despawn_debug_hud)
            .add_systems(Update, (
                update_fps,
                update_ping.run_if(not(in_state(ConnectionState::None))),
                update_game_age.run_if(in_state(OnlineState::Client)),
                update_in_game_time,
                inspector_ui,
            ).run_if(in_state(HudDebugState::Enabled))
                .run_if(in_state(AppState::InGame)));
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
