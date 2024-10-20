use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::*;

mod ui;
mod game;
mod commands;
mod audio;

use commands::{execute_cmds, Command};
use ui::UiPlugin;
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
            RapierDebugRenderPlugin::default(),
            UiPlugin {},
            GamePlugin {},
            SoundPlugin {},
        ))
        .init_state::<AppState>()
        .add_event::<Command>()
        .add_event::<ShareWorld>()
        .add_event::<ReceivedWorld>()
        .add_event::<SpawnPlayer>()
        .add_event::<PlayerAttack>()
        .add_event::<ShareMovement>()
        .add_event::<ShareRotation>()
        .add_event::<MovePlayer>()
        .add_event::<RotatePlayer>()
        .add_event::<PlayerJump>()
        .insert_resource(Settings {
            local_lobby: false,
            music_enabled: false,
            sfx_enabled: true,
            hitboxes_enabled: true,
            lobby_url: "91.108.102.51:9983".to_string(),
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
    AwaitingJoinPermission,
}

#[derive(Resource)]
pub struct Settings {
    local_lobby: bool,
    music_enabled: bool,
    sfx_enabled: bool,
    hitboxes_enabled: bool,
    lobby_url: String,
}

#[derive(Event)]
pub struct ShareWorld;

#[derive(Event)]
pub struct ReceivedWorld(pub String);

#[derive(Event)]
pub struct SpawnPlayer {
    pub name: String,
    pub id: u16,
    pub position: Transform
}

#[derive(Event)]
pub struct PlayerAttack {
    pub player_id: u16,
    pub position: Transform
}

#[derive(Resource)]
pub struct ShareMovementTimer(pub Timer);
#[derive(Event)]
pub struct ShareMovement(pub Vec3);

#[derive(Resource)]
pub struct ShareRotationTimer(pub Timer);
#[derive(Event)]
pub struct ShareRotation(pub Quat);

#[derive(Event)]
pub struct MovePlayer {
    id: u16,
    position: Vec3
}

#[derive(Event)]
pub struct RotatePlayer {
    id: u16,
    rotation: Quat
}

#[derive(Event)]
pub struct PlayerJump(pub u16);
