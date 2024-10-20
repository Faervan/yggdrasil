use bevy::prelude::*;

pub mod components;
mod controll_systems;
mod systems;
mod host_mode;
mod client_mode;

use controll_systems::*;
use systems::*;
use components::*;

use crate::{ui::chat::ChatState, AppState, MovePlayer, PlayerAttack, PlayerJump, ReceivedWorld, RotatePlayer, ShareAttack, ShareJump, ShareMovement, ShareMovementTimer, ShareRotation, ShareRotationTimer, ShareWorld, SpawnPlayer};

use self::{client_mode::load_world, host_mode::share_world};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Player>()
            .register_type::<Health>()
            .register_type::<Npc>()
            .init_state::<OnlineGame>()
            .insert_resource(PlayerName("Jon".to_string()))
            .insert_resource(PlayerId(0))
            .insert_resource(ShareMovementTimer(Timer::from_seconds(0.05, TimerMode::Once)))
            .insert_resource(ShareRotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
            .add_systems(OnEnter(AppState::InGame), (
                setup_light,
                spawn_main_character,
                spawn_camera.after(spawn_main_character),
                spawn_floor,
                spawn_scene.run_if(in_state(OnlineGame::Client)),
                spawn_enemy.run_if(not(in_state(OnlineGame::Client))),
            ))
            .add_systems(Update, (
                rotate_player,
                rotate_camera.before(move_camera),
                zoom_camera.before(move_camera),
                move_player.run_if(not(in_state(ChatState::Open))),
                move_camera.after(move_player),
                respawn_players,
                player_attack,
                move_bullets,
                bullet_hits_attackable,
                animate_walking,
                advance_timers,
                toggle_debug,
                insert_player_components,
                insert_npc_components,
            ).run_if(in_state(AppState::InGame)))
            // Tuple limit is 20, need new call
            .add_systems(Update, (
                share_movement.run_if(on_event::<ShareMovement>()),
                share_rotation.run_if(on_event::<ShareRotation>()),
                share_jump.run_if(on_event::<ShareJump>()),
                share_attack.run_if(on_event::<ShareAttack>()),
                spawn_player.run_if(on_event::<SpawnPlayer>()),
                spawn_bullets.run_if(on_event::<PlayerAttack>()),
                move_other_players.run_if(on_event::<MovePlayer>()),
                rotate_other_players.run_if(on_event::<RotatePlayer>()),
                other_players_jump.run_if(on_event::<PlayerJump>()),
            ).run_if(not(in_state(OnlineGame::None))))
            .add_systems(Update, (
                return_to_menu.run_if(not(in_state(ChatState::Open))),
            ).run_if(in_state(AppState::InGame)).run_if(in_state(OnlineGame::None)))
            .add_systems(Update, (
                return_to_lobby.run_if(not(in_state(ChatState::Open))),
            ).run_if(in_state(AppState::InGame)).run_if(not(in_state(OnlineGame::None))))
            .add_systems(Update, (
                load_world.run_if(on_event::<ReceivedWorld>())
            ).run_if(in_state(OnlineGame::Client)))
            .add_systems(Update, (
                share_world.run_if(on_event::<ShareWorld>())
            ).run_if(in_state(OnlineGame::Host)))
            .add_systems(OnExit(AppState::InGame), (
                despawn_all_entities,
                set_online_state_none,
            ));
    }
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum OnlineGame {
    #[default]
    None,
    Host,
    Client,
}

#[derive(Resource)]
pub struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

#[derive(Resource)]
pub struct PlayerName(pub String);

#[derive(Resource)]
pub struct PlayerId(pub u16);

#[derive(Resource)]
struct WorldScene(Handle<DynamicScene>);
