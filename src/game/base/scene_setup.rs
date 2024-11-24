use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use super::components::GameComponentParent;

pub fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dimension = 20.;
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(dimension))),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0., -1., 0.),
            ..default()
        },
        GameComponentParent {},
        RigidBody::Fixed {},
        Collider::cuboid(dimension, 1., dimension),
        CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_3),
    ));
}
