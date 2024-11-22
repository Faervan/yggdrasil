use bevy::prelude::*;
use bevy_rapier3d::prelude::{AdditionalMassProperties, Collider, CollisionGroups, GravityScale, Group, LockedAxes, RigidBody, Velocity};

use crate::{game::online::events::{DespawnPlayer, SpawnPlayer}, ui::chat::ChatState, AppState};

use crate::game::base::{camera::CameraState, components::{Follow, GameComponentParent, GlobalUiPosition, Health, MainCharacter, Player}, resources::{Animations, PlayerId, PlayerName}};
use player_ctrl::{move_player, player_attack, rotate_eagle_player, rotate_normal_player};

use super::components::{AnimationState, GameComponent};

pub mod player_ctrl;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerName("Jon".to_string()))
            .insert_resource(PlayerId(0))
            .add_systems(OnEnter(AppState::InGame), spawn_main_character)
            .add_systems(Update, (
                rotate_eagle_player.run_if(in_state(CameraState::Eagle)),
                rotate_normal_player.run_if(in_state(CameraState::Normal)),
                move_player.run_if(not(in_state(ChatState::Open))),
                respawn_players,
                player_attack,
                insert_player_components,
            ).run_if(in_state(AppState::InGame)));
    }
}

pub fn spawn_main_character(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset: Res<AssetServer>,
    player_name: Res<PlayerName>,
    player_id: Res<PlayerId>,
) {
    let mut graph = AnimationGraph::new();

    let animations = graph.add_clips(
        [
            GltfAssetLabel::Animation(0),
            GltfAssetLabel::Animation(1),
            GltfAssetLabel::Animation(2),
        ].into_iter()
        .map(|path| asset.load(path.from_asset("embedded://sprites/undead_mage.glb"))),
        1.,
        graph.root
    ).collect();

    commands.insert_resource(Animations {
        animations,
        graph: graphs.add(graph),
    });

    commands.spawn((
        MainCharacter,
        Player {
            mc: true,
            base_velocity: 4.,
            name: player_name.0.clone(),
            id: player_id.0
        },
        Health {
            value: 5
        },
        VisibilityBundle {
            visibility: Visibility::Visible,
            ..default()
        },
        TransformBundle::from_transform(Transform::from_xyz(0., 10., 0.)),
    ));
}

pub fn spawn_player(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnPlayer>,
) {
    let spawn_event = event_reader.read().next().expect("All according to plan of course");

    commands.spawn((
        Player {
            mc: false,
            base_velocity: 4.,
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
    player_query: Query<(Entity, &Player, &GlobalUiPosition)>,
    mut event_reader: EventReader<DespawnPlayer>,
) {
    for event in event_reader.read().into_iter() {
        player_query.iter().find(|(_, p, _)| p.id == event.0).map(|(entity, _, node)| {
            commands.entity(node.node_entity).despawn_recursive();
            commands.entity(entity).despawn_recursive();
        });
    }
}

pub fn insert_player_components(
    mut commands: Commands,
    asset: Res<AssetServer>,
    player_query: Query<(Entity, &Player, &Transform), Added<Player>>,
    camera_state: Res<State<CameraState>>,
) {
    for (player_entity, player, player_pos) in &player_query {
        let player_mesh = asset.load(GltfAssetLabel::Scene(0).from_asset("embedded://sprites/undead_mage.glb"));
        let node_entity = commands.spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Px(player_pos.translation.x),
                        top: Val::Px(player_pos.translation.y-150.),
                        right: Val::ZERO,
                        bottom: Val::ZERO
                    },
                    ..default()
                },
                ..default()
            },
            Follow { entity: player_entity },
            GameComponentParent {},
        )).with_children(|p| {
            p.spawn((
                TextBundle::from_section(player.name.clone(), TextStyle {font_size: 50., color: Color::BLACK, ..default()}),
            ));
        }).id();
        commands.entity(player_entity).insert((
            SceneBundle {
                scene: player_mesh,
                visibility: match *camera_state.get() == CameraState::Normal && player.mc {
                    true => Visibility::Hidden,
                    false => Visibility::Visible
                },
                ..default()
            },
            AnimationState::Idle,
            RigidBody::Dynamic,
            Collider::cylinder(1., 0.25),
            GravityScale(9.81),
            AdditionalMassProperties::Mass(1.),
            Velocity::zero(),
            CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
            (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
            GameComponent,
            GlobalUiPosition {
                pos: Vec2::ZERO,
                node_entity
            },
        ));
    }
}

pub fn respawn_players(
    mut players: Query<(&mut Transform, &mut Health, &mut Velocity), With<Player>>,
) {
    for (mut player, mut health, mut body) in players.iter_mut() {
        if player.translation.y < -100. || health.value < 1 {
            *player = Transform::from_xyz(0., 20., 0.);
            *body = Velocity::zero();
            if health.value < 1 {
                health.value = 5;
            }
        }
    }
}
