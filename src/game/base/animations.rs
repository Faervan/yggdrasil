use std::time::Duration;

use bevy::prelude::*;

use super::resources::Animations;

pub fn animate_walking(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<Handle<AnimationGraph>>>,
    animations: Res<Animations>,
) {
    for (entity, mut player) in players.iter_mut() {
        let mut transitions = AnimationTransitions::new();
        transitions.play(&mut player, animations.animations[0], Duration::ZERO).repeat();
        transitions.play(&mut player, animations.animations[1], Duration::ZERO).repeat();

        commands.entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
        //commands.entity(entity).insert(animations.graph.clone());
        //player.play(animations.animations[0]).repeat();
    }
}
