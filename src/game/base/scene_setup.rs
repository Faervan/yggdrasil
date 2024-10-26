use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use super::components::GameComponentParent;

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
