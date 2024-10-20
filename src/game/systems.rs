use std::f32::consts::PI;

use bevy::{color::palettes::css::BLUE, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_rapier3d::prelude::{LockedAxes, *};
use ysync::{TcpFromClient, UdpPackage, YPosition, YRotation, YTranslation};
use crate::{ui::{chat::ChatState, lobby::LobbySocket}, AppState, DespawnPlayer, PlayerAttack, ShareAttack, ShareMovement, ShareMovementTimer, ShareRotation, ShareRotationTimer, SpawnPlayer};

use super::{components::{Camera, *}, Animations, OnlineGame, PlayerId, PlayerName, WorldScene};

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

pub fn spawn_main_character(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset: Res<AssetServer>,
    player_name: Res<PlayerName>,
    player_id: Res<PlayerId>,
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
    commands.spawn((
        MainCharacter,
        Player {
            base_velocity: 10.,
            name: player_name.0.clone(),
            id: player_id.0
        },
        Health {
            value: 5
        },
        TransformBundle::from_transform(Transform::from_xyz(0., 10., 0.).with_scale(Vec3::new(0.4, 0.4, 0.4))),
    ));
}

pub fn spawn_player(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset: Res<AssetServer>,
    mut event_reader: EventReader<SpawnPlayer>,
) {
    let spawn_event = event_reader.read().next().expect("All according to plan of course");
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
    commands.spawn((
        Player {
            base_velocity: 10.,
            name: spawn_event.name.clone(),
            id: spawn_event.id
        },
        Health {
            value: 5
        },
        TransformBundle::from_transform(spawn_event.position),
    ));
}

pub fn despawn_players(
    mut commands: Commands,
    player_query: Query<(Entity, &Player)>,
    mut event_reader: EventReader<DespawnPlayer>,
) {
    for event in event_reader.read().into_iter() {
        player_query.iter().find(|(_, p)| p.id == event.0).map(|(entity, _)| {
            commands.entity(entity).despawn_recursive();
        });
    }
}

pub fn insert_player_components(
    mut commands: Commands,
    asset: Res<AssetServer>,
    player_query: Query<(Entity, &Transform), Added<Player>>,
) {
    for (player, pos) in player_query.iter() {
        println!("Transform of Player is: {pos:?}");
        let player_mesh: Handle<Scene> = asset.load("embedded://sprites/player3.glb#Scene0");
        commands.entity(player).insert((
            player_mesh,
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

pub fn spawn_scene(
    mut commands: Commands,
    world_scene: Res<WorldScene>,
) {
    println!("spawning scene");
    commands.spawn(DynamicSceneBundle {
        scene: world_scene.0.clone(),
        ..default()
    });
    commands.remove_resource::<WorldScene>();
}

pub fn spawn_enemy(
    mut commands: Commands,
) {
    commands.spawn((
        Health {
            value: 5
        },
        Npc,
        TransformBundle::from_transform(Transform::from_xyz(30., 10., 0.).with_scale(Vec3::new(0.4, 0.4, 0.4))),
    ));
}

pub fn insert_npc_components(
    mut commands: Commands,
    asset: Res<AssetServer>,
    npc_query: Query<Entity, Added<Npc>>,
) {
    for npc in npc_query.iter() {
        let enemy_mesh: Handle<Scene> = asset.load("embedded://sprites/player3.glb#Scene0");
        commands.entity(npc).insert((
            enemy_mesh,
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
}

pub fn respawn_players(
    mut players: Query<(&mut Transform, &mut Health, &mut Velocity), With<Player>>,
) {
    for (mut player, mut health, mut body) in players.iter_mut() {
        if player.translation.y < -100. || health.value < 1 {
            *player = Transform::from_xyz(0., 20., 0.).with_scale(Vec3::new(0.4, 0.4, 0.4));
            *body = Velocity::zero();
            if health.value < 1 {
                health.value = 5;
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

pub fn spawn_bullets(
    mut attack_event: EventReader<PlayerAttack>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let event = attack_event.read().next().expect("No attack event huh");
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.7).mesh()),
            material: materials.add(StandardMaterial::from_color(BLUE)),
            transform: event.position,
            ..default()
        },
        Bullet {
            origin: event.position.translation,
            range: 40.,
            velocity: 40.,
            shooter: event.player_id,
        },
        GameComponentParent {},
    ));
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
    mut players: Query<(&mut Health, &Transform, &Player, Entity)>,
    mut attackables: Query<(&mut Health, &Transform, Entity), Without<Player>>,
    bullets: Query<(&Transform, Entity, &Bullet)>,
    mut commands: Commands,
) {
    for (bullet_pos, bullet_id, bullet) in bullets.iter() {
        for (mut health, player_pos, player, _entity) in players.iter_mut() {
            if bullet_pos.translation.distance(player_pos.translation) <= 2. && bullet.shooter != player.id {
                commands.entity(bullet_id).despawn();
                health.value -= 1;
                //if health.value == 0 {
                //    commands.entity(entity).despawn();
                //}
            }
        }
        for (mut health, attackable_pos, entity) in attackables.iter_mut() {
            if bullet_pos.translation.distance(attackable_pos.translation) <= 2. {
                commands.entity(bullet_id).despawn();
                health.value -= 1;
                if health.value == 0 {
                    commands.entity(entity).despawn();
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

pub fn set_online_state_none(mut next_state: ResMut<NextState<OnlineGame>>) {
    next_state.set(OnlineGame::None);
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
    mut socket: ResMut<LobbySocket>,
    online_state: Res<State<OnlineGame>>,
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        if *online_state.get() == OnlineGame::Host {
            let _ = socket.socket.tcp_send.send(TcpFromClient::GameDeletion);
        } else {
            let _ = socket.socket.tcp_send.send(TcpFromClient::GameExit);
        }
        next_state.set(AppState::MultiplayerLobby(crate::LobbyState::InLobby));
        socket.socket.game_id = None;
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

pub fn advance_timers(
    mut share_movement: ResMut<ShareMovementTimer>,
    mut share_rotation: ResMut<ShareRotationTimer>,
    time: Res<Time>,
) {
    share_movement.0.tick(time.delta());
    share_rotation.0.tick(time.delta());
}

pub fn share_movement(
    socket: Res<LobbySocket>,
    mut movement_event: EventReader<ShareMovement>,
) {
    let event = movement_event.read().next().expect("All according to plan of course");
    let _ = socket.socket.udp_send.send(UdpPackage::Move(YTranslation::from(event.0)));
}

pub fn share_rotation(
    socket: Res<LobbySocket>,
    mut rotation_event: EventReader<ShareRotation>,
) {
    let event = rotation_event.read().next().expect("All according to plan of course");
    let _ = socket.socket.udp_send.send(UdpPackage::Rotate(YRotation::from(event.0)));
}

pub fn share_jump(
    remote: Res<LobbySocket>,
) {
    let _ = remote.socket.udp_send.send(UdpPackage::Jump);
}

pub fn share_attack(
    socket: Res<LobbySocket>,
    mut attack_event: EventReader<ShareAttack>,
) {
    let event = attack_event.read().next().expect("All according to plan of course");
    let _ = socket.socket.udp_send.send(UdpPackage::Attack(YPosition::from(event.0)));
}
