use bevy::prelude::*;
use bevy_rapier3d::prelude::{AdditionalMassProperties, Collider, CollisionGroups, GravityScale, Group, LockedAxes, RigidBody, Velocity};

use super::components::{AnimationState, Follow, GameComponent, GameComponentParent, GlobalUiPosition, Health, Npc};

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
        TransformBundle::from_transform(Transform::from_xyz(8., 10., 0.)),
    ));
}

pub fn insert_npc_components(
    mut commands: Commands,
    asset: Res<AssetServer>,
    npc_query: Query<(Entity, &Transform), Added<Npc>>,
) {
    for (npc, npc_pos) in &npc_query {
        let enemy_mesh: Handle<Scene> = asset.load(GltfAssetLabel::Scene(0).from_asset("embedded://sprites/undead_mage.glb"));
        let node_entity = commands.spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Px(npc_pos.translation.x),
                        top: Val::Px(npc_pos.translation.y-150.),
                        right: Val::ZERO,
                        bottom: Val::ZERO
                    },
                    ..default()
                },
                ..default()
            },
            Follow { entity: npc },
            GameComponentParent {},
        )).with_children(|p| {
            p.spawn((
                TextBundle::from_section("NPC", TextStyle {font_size: 50., color: Color::BLACK, ..default()}),
            ));
        }).id();
        commands.entity(npc).insert((
            enemy_mesh,
            AnimationState::Idle,
            RigidBody::Dynamic {},
            Collider::cylinder(1., 0.25),
            GravityScale(9.81),
            AdditionalMassProperties::Mass(10.),
            Velocity::zero(),
            CollisionGroups::new(Group::GROUP_3, Group::GROUP_2),
            (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
            GameComponent {},
            GlobalUiPosition {
                pos: Vec2::ZERO,
                node_entity
            },
        ));
    }
}
