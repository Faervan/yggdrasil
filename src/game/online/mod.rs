use bevy::prelude::*;
use client::load_world;
use events::{DespawnPlayer, MovePlayer, PlayerAttack, PlayerJump, ReceivedWorld, RotatePlayer, ShareAttack, ShareJump, ShareMovement, ShareRotation, ShareWorld, SpawnPlayer};
use host::share_world;
use receive_events::{move_other_players, other_players_jump, rotate_other_players};
use resource::{ShareMovementTimer, ShareRotationTimer};
use share_events::{share_attack, share_jump, share_movement, share_rotation};
use ysync::TcpFromClient;

use crate::{ui::{chat::ChatState, lobby::LobbySocket}, AppState};

use super::base::{components::{Health, Npc, Player}, players::{despawn_players, spawn_player}, projectiles::spawn_bullets, resources::{GameAge, WorldScene}};

pub mod events;
pub mod resource;
mod client;
mod host;
mod share_events;
mod receive_events;

pub struct GameOnlinePlugin;

impl Plugin for GameOnlinePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Player>()
            .register_type::<Health>()
            .register_type::<Npc>()
            .register_type::<GameAge>()
            .init_state::<OnlineGame>()
            .insert_resource(ShareMovementTimer(Timer::from_seconds(0.05, TimerMode::Once)))
            .insert_resource(ShareRotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
            .add_systems(OnEnter(AppState::InGame), (
                spawn_scene,
            ).run_if(in_state(OnlineGame::Client)))
            .add_systems(Update, (
                share_movement.run_if(on_event::<ShareMovement>()),
                share_rotation.run_if(on_event::<ShareRotation>()),
                share_jump.run_if(on_event::<ShareJump>()),
                share_attack.run_if(on_event::<ShareAttack>()),
                spawn_player.run_if(on_event::<SpawnPlayer>()),
                despawn_players.run_if(on_event::<DespawnPlayer>()),
                spawn_bullets.run_if(on_event::<PlayerAttack>()),
                move_other_players.run_if(on_event::<MovePlayer>()),
                rotate_other_players.run_if(on_event::<RotatePlayer>()),
                other_players_jump.run_if(on_event::<PlayerJump>()),
            ).run_if(not(in_state(OnlineGame::None))))
            .add_systems(Update, return_to_lobby.run_if(not(in_state(ChatState::Open))).run_if(not(in_state(OnlineGame::None))))
            .add_systems(Update, load_world.run_if(on_event::<ReceivedWorld>()).run_if(in_state(OnlineGame::Client)))
            .add_systems(Update, share_world.run_if(on_event::<ShareWorld>()).run_if(in_state(OnlineGame::Host)))
            .add_systems(OnExit(AppState::InGame), set_online_state_none);
    }
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum OnlineGame {
    #[default]
    None,
    Host,
    Client,
}

pub fn spawn_scene(
    mut commands: Commands,
    world_scene: Res<WorldScene>,
) {
    println!("spawning scene");
    commands.spawn(DynamicSceneBundle {
        scene: world_scene.0.clone(),
        ..default()
    });
    commands.remove_resource::<WorldScene>();
}

pub fn return_to_lobby(
    mut remote: ResMut<LobbySocket>,
    online_state: Res<State<OnlineGame>>,
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if *online_state.get() == OnlineGame::Host {
            let _ = remote.socket.tcp_send.send(TcpFromClient::GameDeletion);
        } else {
            let _ = remote.socket.tcp_send.send(TcpFromClient::GameExit);
        }
        next_state.set(AppState::MultiplayerLobby(crate::LobbyState::InLobby));
        remote.socket.game_id = None;
    }
}

pub fn set_online_state_none(mut next_state: ResMut<NextState<OnlineGame>>) {
    next_state.set(OnlineGame::None);
}
