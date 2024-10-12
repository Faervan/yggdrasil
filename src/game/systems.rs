use std::f32::consts::PI;

use bevy::{color::palettes::css::BLUE, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_rapier3d::prelude::{LockedAxes, *};
use ysync::client::TcpPackage;
use crate::{ui::{chat::ChatState, lobby::LobbySocket}, AppState};

use super::{components::{Camera, *}, Animations, OnlineGame, PlayerName};

pub fn setup_light(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
   ambient_light.brightness = 150.;
   commands.spawn((
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
           },
            GameComponentParent {},
    ));
    commands.spawn((
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
        },
        GameComponentParent {},
    ));
}

pub fn spawn_player(
    mut commands: Commands,
    asset: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    player_name: Res<PlayerName>,
) {
    let mut graph = AnimationGraph::new();
    let graph_handle = graphs.add(graph.clone());
    commands.insert_resource(Animations {
        animations: graph.add_clips(
            [
                asset.load("embedded://sprites/player3.glb#Animation2"),
                asset.load("embedded://sprites/player3.glb#Animation3")
            ],
            1.0,
            graph.root).collect(),
        graph: graph_handle,
    });
    let player_mesh = asset.load("embedded://sprites/player3.glb#Scene0");
    commands.spawn((
        MainCharacter,
        Player {
            base_velocity: 10.,
            name: player_name.0.clone(),
        },
        Health {
            value: 5
        },
        SceneBundle {
            scene: player_mesh,
            transform: Transform::from_xyz(0., 10., 0.).with_scale(Vec3::new(0.4, 0.4, 0.4)),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cylinder(10., 2.),
        GravityScale(9.81),
        AdditionalMassProperties::Mass(10.),
        Velocity::zero(),
        CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
        (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
        GameComponentParent,
    ));
}

pub fn spawn_camera(
    mut commands: Commands,
    player: Query<&Transform, With<MainCharacter>>,
) {
    let player_pos = player.get_single().unwrap().translation;
    let direction = Vec3::new(0., 30., 20.);
    let distance = 25.;
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
        },
        GameComponentParent {},
    ));
}

pub fn spawn_floor(
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
        Collider::cuboid(dimension, 0.001, dimension),
        GameComponentParent {},
        CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_3),
    ));
}

pub fn spawn_enemy(
    mut commands: Commands,
    asset: Res<AssetServer>,
) {
    let enemy_mesh = asset.load("embedded://sprites/player3.glb#Scene0");
    commands.spawn((
        Health {
            value: 5
        },
        SceneBundle {
            scene: enemy_mesh,
            transform: Transform::from_xyz(30., 10., 0.).with_scale(Vec3::new(0.4, 0.4, 0.4)),
            ..default()
        },
        Npc,
        RigidBody::Dynamic {},
        Collider::cylinder(10., 2.),
        GravityScale(9.81),
        AdditionalMassProperties::Mass(10.),
        Velocity::zero(),
        CollisionGroups::new(Group::GROUP_3, Group::GROUP_2),
        (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
        GameComponentParent {},
    ));
}

pub fn respawn_players(
    mut players: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    for (mut player, mut body) in players.iter_mut() {
        if player.translation.y < -100. {
            *player = Transform::from_xyz(0., 10., 0.).with_scale(Vec3::new(0.3, 0.3, 0.3));
            *body = Velocity::zero();
        }
    }
}

pub fn player_attack(
    player: Query<(&Transform, Entity), With<MainCharacter>>,
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chat_state: Res<State<ChatState>>,
    mut next_state: ResMut<NextState<ChatState>>,
) {
    if input.just_pressed(MouseButton::Left) {
        if *chat_state.get() == ChatState::Open {
            next_state.set(ChatState::Closed);
        }
        if let Ok((player_pos, player_id)) = player.get_single() {
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
                    shooter: player_id,
                },
                GameComponentParent {},
            ));
        }
    }
}

pub fn move_bullets(
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

pub fn bullet_hits_attackable(
    mut attackables: Query<(&mut Health, &Transform, Entity)>,
    bullets: Query<(&Transform, Entity, &Bullet)>,
    mut commands: Commands,
) {
    for (bullet_pos, bullet_id, bullet) in bullets.iter() {
        for (mut health, attackable_pos, attackable_id) in attackables.iter_mut() {
            if bullet_pos.translation.distance(attackable_pos.translation) <= 2. && bullet.shooter != attackable_id {
                commands.entity(bullet_id).despawn();
                health.value -= 1;
                if health.value == 0 {
                    commands.entity(attackable_id).despawn();
                }
            }
        }
    }
}

pub fn despawn_all_entities(
    mut commands: Commands,
    entities: Query<Entity, With<GameComponentParent>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn return_to_menu(
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

pub fn return_to_lobby(
    socket: Res<LobbySocket>,
    online_state: Res<State<OnlineGame>>,
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if *online_state.get() == OnlineGame::Host {
            let _ = socket.socket.tcp_send.send(TcpPackage::GameDeletion);
        } else {
            let _ = socket.socket.tcp_send.send(TcpPackage::GameExit);
        }
        next_state.set(AppState::MultiplayerLobby(crate::LobbyState::InLobby));
    }
}

pub fn animate_walking(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<Handle<AnimationGraph>>>,
    animations: Res<Animations>,
) {
    for (entity, mut player) in players.iter_mut() {
        /*let mut transitions = AnimationTransitions::new();
        transitions.play(&mut player, animations.animations[0], Duration::ZERO).repeat();
        transitions.play(&mut player, animations.animations[1], Duration::ZERO).repeat();

        commands.entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);*/
        commands.entity(entity).insert(animations.graph.clone());
        player.play(animations.animations[0]).repeat();
    }
}