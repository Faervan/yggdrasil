use bevy::{color::palettes::css::BLUE, prelude::*};
use bevy_rapier3d::prelude::Velocity;

use crate::{game::online::{events::{ShareAttack, ShareJump, ShareMovement, ShareRotation}, resource::{ShareMovementTimer, ShareRotationTimer}}, ui::chat::ChatState};

use super::components::{Bullet, GameComponentParent, MainCharacter, Player};

pub fn rotate_eagle_player(
    mut player: Query<&mut Transform, With<MainCharacter>>,
    camera: Query<&Transform, (With<Camera>, Without<MainCharacter>)>,
    window: Query<&Window>,
    mut share_timer: ResMut<ShareRotationTimer>,
    mut share_event: EventWriter<ShareRotation>,
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
            if share_timer.0.finished() {
                share_event.send(ShareRotation(player.rotation));
                share_timer.0.reset();
            }
        }
    }
}

pub fn move_player(
    mut player: Query<(&mut Transform, &Player), With<MainCharacter>>,
    mut player_velocity: Query<&mut Velocity, With<MainCharacter>>,
    camera: Query<&Transform, (With<Camera>, Without<MainCharacter>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut share_timer: ResMut<ShareMovementTimer>,
    mut share_movement: EventWriter<ShareMovement>,
    mut share_jump: EventWriter<ShareJump>,
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
                if share_timer.0.finished() {
                    share_movement.send(ShareMovement(player_pos.translation));
                    share_timer.0.reset();
                }
            }
        }
        if input.just_pressed(KeyCode::Space) {
            if let Ok(mut player_velocity) = player_velocity.get_single_mut() {
                if player_pos.translation.y <= 5. && player_pos.translation.y >= 0. {
                    player_velocity.linvel = Vec3::new(0., 40., 0.);
                    player_velocity.angvel = Vec3::ZERO;
                    share_jump.send(ShareJump);
                }
            }
        }
    }
}

pub fn player_attack(
    player: Query<(&Player, &Transform), With<MainCharacter>>,
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chat_state: Res<State<ChatState>>,
    mut next_state: ResMut<NextState<ChatState>>,
    mut share_event: EventWriter<ShareAttack>,
) {
    if input.just_pressed(MouseButton::Left) {
        if *chat_state.get() == ChatState::Open {
            next_state.set(ChatState::Closed);
        }
        if let Ok((player, player_pos)) = player.get_single() {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.7).mesh()),
                    material: materials.add(StandardMaterial::from_color(BLUE)),
                    transform: *player_pos,
                    ..default()
                },
                Bullet {
                    origin: player_pos.translation,
                    range: 40.,
                    velocity: 40.,
                    shooter: player.id,
                },
                GameComponentParent {},
            ));
            share_event.send(ShareAttack(*player_pos));
        }
    }
}
