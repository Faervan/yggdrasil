use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, pbr::CascadeShadowConfigBuilder, prelude::*};

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(
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
        )
        .add_systems(Startup, (
                setup_light,
                spawn_player,
                spawn_camera.after(spawn_player),
                spawn_floor,
            ))
        .add_systems(Update, (
                close_on_esc,
                rotate_player,
                rotate_camera,
                move_player,
            ))
        .run();
}

#[derive(Component)]
struct Player {
    base_velocity: f32,
}

#[derive(Component)]
struct Camera;

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

fn setup_light(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
   ambient_light.brightness = 100.0;
   commands.spawn(
       DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: light_consts::lux::OVERCAST_DAY,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.),
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            cascade_shadow_config: CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .into(),
            ..default()
           }
       );
}

fn spawn_player(
    mut commands: Commands,
    asset: Res<AssetServer>,
) {
    let player_mesh = asset.load("sprites/player.glb#Scene0");
    commands.spawn((
        Player {
            base_velocity: 10.
        },
        SceneBundle {
            scene: player_mesh,
            transform: Transform::from_xyz(0., 3., 0.),
            ..default()
        }
    ));
}

fn spawn_camera(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
) {
    let player_pos = player.get_single().unwrap().translation;
    let camera_transform = Transform::from_xyz(player_pos.x, player_pos.y + 8., player_pos.z + 20.)
        .with_rotation(Quat::from_axis_angle(Vec3::new(0., 1., 0.), 0.));
    commands.spawn((
        Camera {},
        Camera3dBundle {
            projection: PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }.into(),
            transform: camera_transform,
            ..default()
        }
    ));
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.))),
        material: materials.add(Color::WHITE),
        ..default()
    });
}

fn rotate_player(
    mut player: Query<&mut Transform, With<Player>>,
    window: Query<&Window>,
) {
    if let Some(cursor_pos) = window.get_single().unwrap().cursor_position() {
        if let Ok(mut player) = player.get_single_mut() {
            //let angle_to_cursor = cursor_pos.angle_between(Vec2::new(960., 540.));
            //println!("angle: {angle_to_cursor:?}\n cursor_pos: {cursor_pos:?}");
            //player.rotation = Quat::from_rotation_y(- angle_to_cursor);
            let player_up = player.up();
            let target = Vec3::new(
                player.translation.x + cursor_pos.x - 960.,
                player.translation.y,
                player.translation.z + cursor_pos.y - 540.
            );
            player.look_at(target, player_up);
        }
    }
}

fn rotate_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.pressed(MouseButton::Right) {
        let mut camera = camera.single_mut();
        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.003;
            let pitch = -motion.delta.y * 0.002;
            // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
            camera.rotate_y(yaw);
            camera.rotate_local_x(pitch);
        }
    }
}

fn move_player(
    mut player: Query<(&mut Transform, &Player), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (w, a, s, d) = (input.pressed(KeyCode::KeyW), input.pressed(KeyCode::KeyA), input.pressed(KeyCode::KeyS), input.pressed(KeyCode::KeyD));
    if w || a || s || d {
        if let Ok(mut camera_pos) = camera.get_single_mut() {
            if let Ok((mut player_pos, player)) = player.get_single_mut() {
                let mut direction = Vec3::ZERO;
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
                player_pos.translation += direction.normalize_or_zero() * player.base_velocity * time.delta_seconds();
                camera_pos.translation = Vec3::new(
                    player_pos.translation.x,
                    player_pos.translation.y + 8.,
                    player_pos.translation.z + 20.
                );
            }
        }
    }
}
