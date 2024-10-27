use bevy::{input::mouse::{MouseMotion, MouseWheel}, prelude::*, window::CursorGrabMode};

use crate::AppState;

use super::{components::{GameCamera, GameComponentParent, MainCharacter}, player_ctrl::move_player, players::spawn_main_character};

pub const MAX_CAMERA_DISTANCE: f32 = 50.;
pub const MIN_CAMERA_DISTANCE: f32 = 5.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<CameraState>()
            .add_systems(OnEnter(AppState::InGame),
                spawn_camera.after(spawn_main_character)
            )
            .add_systems(Update, (
                rotate_camera,
                zoom_camera,
                move_camera.after(move_player),
                toggle_camera_mode,
            ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(States, Clone, Default, PartialEq, Eq, Hash, Debug)]
enum CameraState {
    FirstPerson,
    #[default]
    ThirdPerson
}

fn spawn_camera(
    mut commands: Commands,
    player: Query<&Transform, With<MainCharacter>>,
) {
    let player_pos = player.get_single().unwrap().translation;
    let direction = Vec3::new(0., 30., 20.);
    let distance = 25.;
    let camera_transform = Transform::from_translation(player_pos + direction.normalize() * distance).looking_at(player_pos, Vec3::Y);
    commands.spawn((
        GameCamera {
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
}

fn rotate_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera: Query<(&Transform, &mut GameCamera)>,
    mut window: Query<&mut Window>,
    player: Query<&Transform, (With<MainCharacter>, Without<Camera>)>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.pressed(MouseButton::Right) {
        let mut window = window.get_single_mut().unwrap();
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        let (camera_pos, mut camera) = camera.single_mut();
        let player = player.single().translation;
        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.01;
            camera.direction = Quat::from_rotation_y(yaw) * (camera_pos.translation - player);
        }
    } else if input.just_released(MouseButton::Right) {
        let mut window = window.get_single_mut().unwrap();
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn zoom_camera(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera: Query<&mut GameCamera>,
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

fn move_camera(
    player: Query<&Transform, With<MainCharacter>>,
    mut camera: Query<(&mut Transform, &GameCamera), Without<MainCharacter>>,
) {
    if let Ok(player) = player.get_single() {
        let (mut camera_pos, camera) = camera.get_single_mut().unwrap();
        *camera_pos = Transform::from_translation(player.translation + camera.direction.normalize() * camera.distance).looking_at(player.translation, Vec3::Y);
    }
}

fn toggle_camera_mode(
    input: Res<ButtonInput<KeyCode>>,
    camera_state: Res<State<CameraState>>,
    mut next_state: ResMut<NextState<CameraState>>,
) {
    if input.just_pressed(KeyCode::KeyV) {
        next_state.set(match camera_state.get() {
            CameraState::FirstPerson => CameraState::ThirdPerson,
            CameraState::ThirdPerson => CameraState::FirstPerson
        });
    }
}
