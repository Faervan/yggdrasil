use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::*;

mod ui;
mod game;
mod commands;
mod audio;

use commands::{execute_cmds, GameCommand};
use ui::{lobby::LobbyState, UiPlugin};
use game::GamePlugin;
use audio::SoundPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "Yggdrasil".into(),
                        name: Some("yggdrasil".into()),
                        resolution: bevy::window::WindowResolution::with_scale_factor_override((1920.0, 1080.0).into(), 1.0),
                        mode: bevy::window::WindowMode::BorderlessFullscreen,
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        enabled_buttons: bevy::window::EnabledButtons { minimize: false, maximize: false, close: false },
                        ..default()
                    }),
                    ..default()
                }
            ),
            EmbeddedAssetPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin {
                enabled: false,
                ..default()
            },
            UiPlugin {},
            GamePlugin {},
            SoundPlugin {},
        ))
        .init_state::<AppState>()
        .add_event::<GameCommand>()
        .insert_resource(Settings {
            local_lobby: false,
            music_enabled: false,
            sfx_enabled: true,
            // default has to be set in Plugin insertion as well
            hitboxes_enabled: false,
            debug_hud_enabled: true,
            lobby_url: "91.108.102.51:9983".to_string(),
        })
        .add_systems(Update, execute_cmds)
        .run();
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum AppState {
    #[default]
    MainMenu,
    Lobby(LobbyState),
    InGame,
}

#[derive(Resource)]
pub struct Settings {
    local_lobby: bool,
    music_enabled: bool,
    sfx_enabled: bool,
    hitboxes_enabled: bool,
    debug_hud_enabled: bool,
    lobby_url: String,
}
