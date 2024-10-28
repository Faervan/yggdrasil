use bevy::{input::mouse::{MouseMotion, MouseWheel}, prelude::*};

use crate::AppState;

use super::{components::{EagleCamera, GameComponentParent, MainCharacter, NormalCamera}, cursor::CursorGrabState, players::{player_ctrl::move_player, spawn_main_character}};

pub const MAX_CAMERA_DISTANCE: f32 = 50.;
pub const MIN_CAMERA_DISTANCE: f32 = 5.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<CameraState>()
            .add_systems(OnEnter(AppState::InGame), (
                spawn_eagle_camera.run_if(in_state(CameraState::Eagle)),
                spawn_normal_camera.run_if(in_state(CameraState::Normal))
            ).after(spawn_main_character))
            .add_systems(OnExit(CameraState::Normal), despawn_cameras)
            .add_systems(OnExit(CameraState::Eagle), despawn_cameras)
            .add_systems(OnEnter(CameraState::Normal), spawn_normal_camera)
            .add_systems(OnEnter(CameraState::Eagle), spawn_eagle_camera.run_if(in_state(AppState::InGame)))
            .add_systems(Update, (
                (
                    rotate_eagle_camera,
                    zoom_eagle_camera,
                    move_eagle_camera.after(move_player)
                ).run_if(in_state(CameraState::Eagle)),
                move_normal_camera.run_if(in_state(CameraState::Normal)),
                toggle_camera_mode,
            ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(States, Clone, Default, PartialEq, Eq, Hash, Debug)]
pub enum CameraState {
    Normal,
    #[default]
    Eagle
}

const PLAYER_EYE_POS: Vec3 = Vec3 {x: 0., y: 6., z: 0.};

fn spawn_eagle_camera(
    mut commands: Commands,
    player: Query<(&Transform, Entity), With<MainCharacter>>,
    mut cursor_state: ResMut<NextState<CursorGrabState>>,
) {
    if let Ok((player_pos, player_entity)) = player.get_single() {
        cursor_state.set(CursorGrabState::Free);
        let direction = Vec3::new(0., 30., 20.);
        let distance = 25.;
        let camera_transform = Transform::from_translation(player_pos.translation + direction.normalize() * distance).looking_at(player_pos.translation, Vec3::Y);
        commands.spawn((
            EagleCamera {
                direction,
                distance,
            },
            Camera3dBundle {
                projection: PerspectiveProjection {
                    fov: 90.0_f32.to_radians(),
                    ..default()
                }.into(),
                transform: camera_transform,
                ..default()
            },
            GameComponentParent {},
        ));
        commands.entity(player_entity).insert(Visibility::Visible);
    }
}

fn spawn_normal_camera(
    mut commands: Commands,
    player: Query<(&Transform, Entity), With<MainCharacter>>,
    mut cursor_state: ResMut<NextState<CursorGrabState>>,
) {
    if let Ok((player_pos, player_entity)) = player.get_single() {
        cursor_state.set(CursorGrabState::Grabbed);
        let mut camera_transform = player_pos.with_scale(Vec3::ONE);
        camera_transform.translation += PLAYER_EYE_POS;
        commands.spawn((
            NormalCamera,
            Camera3dBundle {
                projection: PerspectiveProjection {
                    fov: 90.0_f32.to_radians(),
                    ..default()
                }.into(),
                transform: camera_transform,
                ..default()
            },
            GameComponentParent {},
        ));
        commands.entity(player_entity).insert(Visibility::Hidden);
    }
}

fn rotate_eagle_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera: Query<(&Transform, &mut EagleCamera)>,
    mut cursor_state: ResMut<NextState<CursorGrabState>>,
    player: Query<&Transform, (With<MainCharacter>, Without<Camera>)>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.pressed(MouseButton::Right) {
        cursor_state.set(CursorGrabState::Grabbed);
        let (camera_pos, mut camera) = camera.single_mut();
        let player = player.single().translation;
        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.01;
            camera.direction = Quat::from_rotation_y(yaw) * (camera_pos.translation - player);
        }
    } else if input.just_released(MouseButton::Right) {
        cursor_state.set(CursorGrabState::Free);
    }
}

fn zoom_eagle_camera(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera: Query<&mut EagleCamera>,
) {
    for scroll in mouse_wheel.read() {
        let mut camera = camera.get_single_mut().unwrap();
        let scroll_factor = 1. - scroll.y / 10.;
        match camera.distance * scroll_factor {
            x if x < MIN_CAMERA_DISTANCE => camera.distance = MIN_CAMERA_DISTANCE,
            x if x > MAX_CAMERA_DISTANCE => camera.distance = MAX_CAMERA_DISTANCE,
            x => camera.distance = x,
        }
    }
}

fn move_eagle_camera(
    player: Query<&Transform, With<MainCharacter>>,
    mut camera: Query<(&mut Transform, &EagleCamera), Without<MainCharacter>>,
) {
    if let Ok(player) = player.get_single() {
        let (mut camera_pos, camera) = camera.get_single_mut().unwrap();
        *camera_pos = Transform::from_translation(player.translation + camera.direction.normalize() * camera.distance).looking_at(player.translation, Vec3::Y);
    }
}

fn move_normal_camera(
    player: Query<&Transform, With<MainCharacter>>,
    mut camera: Query<&mut Transform, (With<NormalCamera>, Without<MainCharacter>)>,
) {
    if let Ok(player) = player.get_single() {
        let mut camera = camera.get_single_mut().unwrap();
        *camera = player.with_scale(Vec3::ONE).with_translation(player.translation + PLAYER_EYE_POS);
    }
}

fn toggle_camera_mode(
    input: Res<ButtonInput<KeyCode>>,
    camera_state: Res<State<CameraState>>,
    mut next_state: ResMut<NextState<CameraState>>,
) {
    if input.just_pressed(KeyCode::KeyV) {
        next_state.set(match camera_state.get() {
            CameraState::Normal => CameraState::Eagle,
            CameraState::Eagle => CameraState::Normal
        });
    }
}

fn despawn_cameras(
    cameras: Query<Entity, With<Camera>>,
    mut commands: Commands
) {
    for camera in cameras.iter() {
        commands.entity(camera).despawn_recursive();
    }
}