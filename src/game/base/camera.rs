use bevy::{input::mouse::{MouseMotion, MouseWheel}, prelude::*, window::CursorGrabMode};

use super::components::{GameCamera, GameComponentParent, MainCharacter};

pub const MAX_CAMERA_DISTANCE: f32 = 50.;
pub const MIN_CAMERA_DISTANCE: f32 = 5.;

pub fn spawn_camera(
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

pub fn rotate_camera(
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

pub fn zoom_camera(
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

pub fn move_camera(
    player: Query<&Transform, With<MainCharacter>>,
    mut camera: Query<(&mut Transform, &GameCamera), Without<MainCharacter>>,
) {
    if let Ok(player) = player.get_single() {
        let (mut camera_pos, camera) = camera.get_single_mut().unwrap();
        *camera_pos = Transform::from_translation(player.translation + camera.direction.normalize() * camera.distance).looking_at(player.translation, Vec3::Y);
    }
}
