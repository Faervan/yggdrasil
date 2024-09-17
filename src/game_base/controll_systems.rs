use bevy::{prelude::*, input::mouse::{MouseMotion, MouseWheel}};
use bevy_rapier3d::prelude::*;

use super::components::{Player, Camera};

const MAX_CAMERA_DISTANCE: f32 = 50.;
const MIN_CAMERA_DISTANCE: f32 = 5.;

pub fn rotate_player(
    mut player: Query<&mut Transform, With<Player>>,
    camera: Query<&Transform, (With<Camera>, Without<Player>)>,
    window: Query<&Window>,
) {
    if let Some(cursor_pos) = window.get_single().unwrap().cursor_position() {
        if let Ok(mut player) = player.get_single_mut() {
            let player_up = player.up();
            let camera = camera.get_single().unwrap();
            let mut target = camera.rotation * Vec3::new(
                player.translation.x + cursor_pos.x - 960.,
                0.,
                player.translation.z + cursor_pos.y - 540.
            );
            target.y = player.translation.y;
            player.look_at(target, player_up);
        }
    }
}

pub fn rotate_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera: Query<(&Transform, &mut Camera), With<Camera>>,
    player: Query<&Transform, (With<Player>, Without<Camera>)>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.pressed(MouseButton::Right) {
        let (camera_pos, mut camera) = camera.single_mut();
        let player = player.single().translation;
        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.03;
            camera.direction = Quat::from_rotation_y(yaw) * (camera_pos.translation - player);
        }
    }
}

pub fn zoom_camera(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera: Query<&mut Camera>,
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

pub fn move_player(
    mut player: Query<(&mut Transform, &Player)>,
    mut player_velocity: Query<&mut Velocity, With<Player>>,
    camera: Query<&Transform, (With<Camera>, Without<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut player_pos, player)) = player.get_single_mut() {
        let (w, a, s, d) = (input.pressed(KeyCode::KeyW), input.pressed(KeyCode::KeyA), input.pressed(KeyCode::KeyS), input.pressed(KeyCode::KeyD));
        if w || a || s || d {
            if let Ok(camera_pos) = camera.get_single() {
                let mut direction = Vec3::ZERO;
                let mut speed_multiplier = 1.;
                if input.pressed(KeyCode::ShiftLeft) {
                    speed_multiplier += 0.8;
                }
                if w {
                    direction += camera_pos.forward().as_vec3();
                }
                if a {
                    direction += camera_pos.left().as_vec3();
                }
                if s {
                    direction += camera_pos.back().as_vec3();
                }
                if d {
                    direction += camera_pos.right().as_vec3();
                }
                direction.y = 0.;
                let movement = direction.normalize_or_zero() * player.base_velocity * speed_multiplier * time.delta_seconds();
                player_pos.translation += movement;
            }
        }
        if input.just_pressed(KeyCode::Space) {
            if let Ok(mut player_velocity) = player_velocity.get_single_mut() {
                if player_pos.translation.y <= 5. && player_pos.translation.y >= 0. {
                    player_velocity.linvel = Vec3::new(0., 40., 0.);
                    player_velocity.angvel = Vec3::ZERO;
                }
            }
        }
    }
}

pub fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut Transform, &Camera), (With<Camera>, Without<Player>)>,
) {
    if let Ok(player) = player.get_single() {
        let (mut camera_pos, camera) = camera.get_single_mut().unwrap();
        *camera_pos = Transform::from_translation(player.translation + camera.direction.normalize() * camera.distance).looking_at(player.translation, Vec3::Y);
    }
}
