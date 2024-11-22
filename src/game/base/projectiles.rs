use bevy::{color::palettes::css::BLUE, prelude::*};

use crate::game::online::events::PlayerAttack;

use super::components::{Bullet, GameComponentParent, GlobalUiPosition, Health, Player};

pub fn spawn_bullets(
    mut attack_event: EventReader<PlayerAttack>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let event = attack_event.read().next().expect("No attack event huh");
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.1).mesh()),
            material: materials.add(StandardMaterial::from_color(BLUE)),
            transform: event.position,
            ..default()
        },
        Bullet {
            origin: event.position.translation,
            range: 14.,
            velocity: 16.,
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
    for (entity, bullet, mut bullet_pos) in &mut bullets {
        let movement = bullet_pos.forward() * bullet.velocity * time.delta_seconds();
        bullet_pos.translation += movement;
        if bullet_pos.translation.distance(bullet.origin) >= bullet.range {
            commands.entity(entity).despawn();
        }
    }
}

pub fn bullet_hits_attackable(
    mut players: Query<(&mut Health, &Transform, &Player, Entity)>,
    mut attackables: Query<(&mut Health, &Transform, Entity, &GlobalUiPosition), Without<Player>>,
    bullets: Query<(&Transform, Entity, &Bullet)>,
    mut commands: Commands,
) {
    for (bullet_pos, bullet_id, bullet) in &bullets {
        for (mut health, player_pos, player, _entity) in &mut players {
            if bullet_pos.translation.distance(player_pos.translation) <= 2. && bullet.shooter != player.id {
                commands.entity(bullet_id).despawn_recursive();
                health.value -= 1;
            }
        }
        for (mut health, attackable_pos, entity, node) in &mut attackables {
            if bullet_pos.translation.distance(attackable_pos.translation) <= 2. {
                commands.entity(bullet_id).despawn();
                health.value -= 1;
                if health.value == 0 {
                    commands.entity(node.node_entity).despawn_recursive();
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
