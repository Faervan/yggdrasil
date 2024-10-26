use bevy::prelude::*;
use bevy_rapier3d::prelude::{AdditionalMassProperties, Collider, CollisionGroups, GravityScale, Group, LockedAxes, RigidBody, Velocity};

use super::components::{Follow, GameComponentParent, GlobalUiPosition, Health, Npc};

pub fn spawn_npc(
    mut commands: Commands,
) {
    commands.spawn((
        Health {
            value: 5
        },
        Npc,
        VisibilityBundle {
            visibility: Visibility::Visible,
            ..default()
        },
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
        let node_entity = commands.spawn((
            NodeBundle::default(),
            Follow { entity: npc },
            GameComponentParent {},
        )).with_children(|p| {
            p.spawn((
                TextBundle::from_section("NPC", TextStyle {font_size: 50., color: Color::BLACK, ..default()}),
            ));
        }).id();
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
            GlobalUiPosition {
                pos: Vec2::ZERO,
                node_entity
            },
        ));
    }
}
