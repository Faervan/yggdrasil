use bevy::{color::palettes::css::BLUE, input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::Velocity;

use crate::{game::{base::{camera::CameraState, components::{AnimationState, WalkDirection}}, online::{events::{ShareAttack, ShareJump, ShareMovement, ShareRotation}, resource::{ShareMovementTimer, ShareRotationTimer}}}, ui::chat::ChatState};

use crate::game::base::components::{Bullet, GameComponentParent, MainCharacter, Player};

pub fn rotate_eagle_player(
    mut player: Query<(&mut Transform, &AnimationState), With<MainCharacter>>,
    camera: Query<&Transform, (With<Camera>, Without<MainCharacter>)>,
    window: Query<&Window>,
    mut share_timer: ResMut<ShareRotationTimer>,
    mut share_event: EventWriter<ShareRotation>,
) {
    if let Some(cursor_pos) = window.get_single().unwrap().cursor_position() {
        if let Ok((mut player, state)) = player.get_single_mut() {
            let player_up = player.up();
            let camera = camera.get_single().unwrap();
            if let AnimationState::Idle = state {
                calc_rotation(&mut player, camera, cursor_pos);
            } else {
                let mut direction = match state {
                    AnimationState::Idle => return,
                    AnimationState::Walk(dir) | AnimationState::Sprint(dir) => match dir {
                        WalkDirection::Forward => *camera.forward(),
                        WalkDirection::Back => *camera.back(),
                        WalkDirection::Left => *camera.left(),
                        WalkDirection::Right => *camera.right(),
                        WalkDirection::ForwardLeft => camera.forward().move_towards(*camera.left(), 0.5),
                        WalkDirection::ForwardRight => camera.forward().move_towards(*camera.right(), 0.5),
                        WalkDirection::BackLeft => camera.back().move_towards(*camera.left(), 0.5),
                        WalkDirection::BackRight => camera.back().move_towards(*camera.right(), 0.5),
                    }
                };
                direction.y = 0.;
                player.look_to(direction, player_up);
            }
            if share_timer.0.finished() {
                share_event.send(ShareRotation(player.rotation));
                share_timer.0.reset();
            }
        }
    }
}

pub fn rotate_normal_player(
    mut player: Query<&mut Transform, With<MainCharacter>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    if let Ok(mut player_pos) = player.get_single_mut() {
        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.003;
            let pitch = -motion.delta.y * 0.002;
            // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
            player_pos.rotate_y(yaw);
            player_pos.rotate_local_x(pitch);
        }
    }
}

pub fn move_player(
    mut player: Query<(&mut Transform, &Player, Entity), With<MainCharacter>>,
    mut player_velocity: Query<&mut Velocity, With<MainCharacter>>,
    camera: Query<&Transform, (With<Camera>, Without<MainCharacter>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut share_timer: ResMut<ShareMovementTimer>,
    mut share_movement: EventWriter<ShareMovement>,
    mut share_jump: EventWriter<ShareJump>,
) {
    if let Ok((mut player_pos, player, entity)) = player.get_single_mut() {
        // Update the AnimationState component if necessary
        if let Some(change) = AnimationState::from_input(&input) {
            commands.entity(entity).insert(change);
        }
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
                if player_pos.translation.y <= 1. && player_pos.translation.y >= 0. {
                    player_velocity.linvel = Vec3::new(0., 30., 0.);
                    player_velocity.angvel = Vec3::ZERO;
                    share_jump.send(ShareJump);
                }
            }
        }
    }
}

pub fn player_attack(
    player: Query<(&Player, &Transform), With<MainCharacter>>,
    camera: Query<&Transform, (With<Camera>, Without<MainCharacter>)>,
    window: Query<&Window>,
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chat_state: Res<State<ChatState>>,
    mut next_state: ResMut<NextState<ChatState>>,
    mut share_event: EventWriter<ShareAttack>,
    camera_state: Res<State<CameraState>>,
) {
    if input.just_pressed(MouseButton::Left) {
        if *chat_state.get() == ChatState::Open {
            next_state.set(ChatState::Closed);
        }
        if let Ok((player, player_pos)) = player.get_single() {
            let mut player_pos = *player_pos;
            if let CameraState::Eagle = camera_state.get() {
                calc_rotation(&mut player_pos, camera.get_single().unwrap(), window.get_single().unwrap().cursor_position().unwrap());
            }
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.1).mesh()),
                    material: materials.add(StandardMaterial::from_color(BLUE)),
                    transform: player_pos,
                    ..default()
                },
                Bullet {
                    origin: player_pos.translation,
                    range: 14.,
                    velocity: 16.,
                    shooter: player.id,
                },
                GameComponentParent {},
            ));
            share_event.send(ShareAttack(player_pos));
        }
    }
}

fn calc_rotation(entity: &mut Transform, camera: &Transform, cursor: Vec2) {
    let mut target = camera.rotation * Vec3::new(
        entity.translation.x + cursor.x - 960.,
        0.,
        entity.translation.z + cursor.y - 540.
    );
    target.y = entity.translation.y;
    entity.look_at(target, entity.up());
}
