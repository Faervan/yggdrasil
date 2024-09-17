use std::f32::consts::PI;

use bevy::{color::palettes::css::BLUE, input::mouse::{MouseMotion, MouseWheel}, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_rapier3d::prelude::*;

const MAX_CAMERA_DISTANCE: f32 = 50.;
const MIN_CAMERA_DISTANCE: f32 = 5.;

fn main() {
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
                spawn_enemy,
            ))
        .add_systems(Update, (
                close_on_esc,
                rotate_player,
                rotate_camera.before(move_camera),
                zoom_camera.before(move_camera),
                move_player,
                move_camera.after(move_player),
                respawn_player,
                player_attack,
                move_bullets,
                bullet_hits_attackable,
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

#[derive(Component)]
struct Bullet {
    origin: Vec3,
    range: f32,
    velocity: f32,
}

#[derive(Component)]
struct Attackable;

#[derive(Component)]
struct Health {
    value: u32,
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
   ambient_light.brightness = 150.;
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
                maximum_distance: 200.0,
                ..default()
            }
            .into(),
            ..default()
           }
    );
    commands.spawn(
        PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                shadows_enabled: true,
                intensity: 100000000.,
                range: 200.,
                ..default()
            },
            transform: Transform::from_xyz(0., 50., 0.),
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
        Health {
            value: 5
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
        Velocity::zero(),
        //Attackable,
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
    let dimension = 100.;
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(dimension))),
            material: materials.add(Color::WHITE),
            ..default()
        },
        RigidBody::Fixed {},
        Collider::cuboid(dimension, 0.1, dimension),
    ));
}

fn spawn_enemy(
    mut commands: Commands,
    asset: Res<AssetServer>,
) {
    let enemy_mesh = asset.load("sprites/player.glb#Scene0");
    commands.spawn((
        Health {
            value: 5
        },
        SceneBundle {
            scene: enemy_mesh,
            transform: Transform::from_xyz(30., 10., 0.),
            ..default()
        },
        RigidBody::Dynamic {},
        Collider::cylinder(4., 2.),
        GravityScale(9.81),
        AdditionalMassProperties::Mass(10.),
        Velocity::zero(),
        Attackable,
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

fn zoom_camera(
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

fn move_player(
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

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut Transform, &Camera), (With<Camera>, Without<Player>)>,
) {
    if let Ok(player) = player.get_single() {
        let (mut camera_pos, camera) = camera.get_single_mut().unwrap();
        *camera_pos = Transform::from_translation(player.translation + camera.direction.normalize() * camera.distance).looking_at(player.translation, Vec3::Y);
    }
}

fn respawn_player(
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut player, mut body)) = player.get_single_mut() {
        if player.translation.y < -100. {
            *player = Transform::from_xyz(0., 10., 0.);
            *body = Velocity::zero();
        }
    }
}

fn player_attack(
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if input.just_pressed(MouseButton::Left) {
        if let Ok(player) = player.get_single() {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.2).mesh()),
                    material: materials.add(StandardMaterial::from_color(BLUE)),
                    transform: *player,
                    ..default()
                },
                Bullet {
                    origin: player.translation,
                    range: 40.,
                    velocity: 40.
                }
            ));
        }
    }
}

fn move_bullets(
    mut bullets: Query<(Entity, &Bullet, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, bullet, mut bullet_pos) in bullets.iter_mut() {
        let movement = bullet_pos.forward() * bullet.velocity * time.delta_seconds();
        bullet_pos.translation += movement;
        if bullet_pos.translation.distance(bullet.origin) >= bullet.range {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_hits_attackable(
    mut attackables: Query<(&mut Health, &Transform, Entity), With<Attackable>>,
    bullets: Query<(&Transform, Entity), With<Bullet>>,
    mut commands: Commands,
) {
    for (bullet_pos, bullet_entity) in bullets.iter() {
        for (mut health, attackable_pos, attackable_entity) in attackables.iter_mut() {
            if bullet_pos.translation.distance(attackable_pos.translation) <= 2. {
                commands.entity(bullet_entity).despawn();
                health.value -= 1;
                if health.value == 0 {
                    commands.entity(attackable_entity).despawn();
                }
            }
        }
    }
}
