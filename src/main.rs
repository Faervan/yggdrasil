use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_rapier3d::prelude::*;

fn main() {
    println!("Hello, world!");
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
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (
                setup_light,
                spawn_player,
                spawn_camera.after(spawn_player),
                spawn_floor,
            ))
        .add_systems(Update, (
                close_on_esc,
                rotate_player,
                rotate_camera.before(move_camera),
                move_player,
                move_camera.after(move_player),
            ))
        .run();
}

#[derive(Component)]
struct Player {
    base_velocity: f32,
}

#[derive(Component)]
struct Camera {
    direction: Vec3,
    distance: f32,
}

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
            transform: Transform::from_xyz(0., 10., 0.),
            ..default()
        },
        RigidBody::Dynamic {},
        Collider::cylinder(4., 2.),
        GravityScale(9.81),
        AdditionalMassProperties::Mass(10.),
    ));
}

fn spawn_camera(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
) {
    let player_pos = player.get_single().unwrap().translation;
    let direction = Vec3::new(0., 8., 20.);
    let distance = 22.;
    let camera_transform = Transform::from_translation(player_pos + direction.normalize() * distance).looking_at(player_pos, Vec3::Y);
    commands.spawn((
        Camera {
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
        }
    ));
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.))),
            material: materials.add(Color::WHITE),
            ..default()
        },
        RigidBody::Fixed {},
        Collider::cuboid(10., 0.1, 10.),
    ));
}

fn rotate_player(
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

fn rotate_camera(
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

fn move_player(
    mut player: Query<(&mut Transform, &Player), With<Player>>,
    camera: Query<&Transform, (With<Camera>, Without<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (w, a, s, d) = (input.pressed(KeyCode::KeyW), input.pressed(KeyCode::KeyA), input.pressed(KeyCode::KeyS), input.pressed(KeyCode::KeyD));
    if w || a || s || d {
        if let Ok(camera_pos) = camera.get_single() {
            if let Ok((mut player_pos, player)) = player.get_single_mut() {
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
    }
}

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut Transform, &Camera), (With<Camera>, Without<Player>)>,
) {
    if let Ok(player) = player.get_single() {
        let (mut camera_pos, camera) = camera.get_single_mut().unwrap();
        *camera_pos = Transform::from_translation(player.translation + camera.direction.normalize() * camera.distance).looking_at(player.translation, Vec3::Y);
    }
}
