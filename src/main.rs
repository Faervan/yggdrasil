use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod ui;
mod game_host;
mod game_client;
mod game_base;

use ui::UiPlugin;
use game_base::GameBasePlugin;
use game_client::GameClientPlugin;

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
            //RapierDebugRenderPlugin::default(),
            UiPlugin {},
            GameBasePlugin {},
            GameClientPlugin {},
        ))
        .init_state::<AppState>()
        .run();
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame(GameSessionType),
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameSessionType {
    #[default]
    Singleplayer,
    GameHost,
    GameClient,
}
