use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod ui;
mod game_host;
mod game_client;
mod game_base;

use game_base::GameBasePlugin;

fn main() {
    App::new()
        .init_state::<AppState>()
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
            GameBasePlugin {},
        ))
        //.add_systems(Startup, (
        //    ))
        .add_systems(Update, (
                close_on_esc,
            ))
        .run();
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame(GameSessionType),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameSessionType {
    Singleplayer,
    GameHost,
    GameClient,
}

fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }
        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
