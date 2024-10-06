use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod ui;
mod game;
mod commands;

use commands::{execute_cmds, Command};
use ui::UiPlugin;
use game::GamePlugin;

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
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            UiPlugin {},
            GamePlugin {},
        ))
        .init_state::<AppState>()
        .add_event::<Command>()
        .insert_resource(Settings {
            local_lobby: false,
        })
        .add_systems(Update, execute_cmds)
        .run();
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum AppState {
    #[default]
    MainMenu,
    MultiplayerLobby(LobbyState),
    InGame,
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum LobbyState {
    #[default]
    ConSelection,
    InLobby,
}

#[derive(Resource)]
pub struct Settings {
    local_lobby: bool,
}
