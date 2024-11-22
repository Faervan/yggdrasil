use std::time::Duration;

use bevy::prelude::*;

use super::{components::AnimationState, resources::Animations};

pub fn setup_animation(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();
        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

pub fn animate(
    animations: Res<Animations>,
    changed: Query<(&Children, &AnimationState), Changed<AnimationState>>,
    mut players: Query<(&Parent, &mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (children, animation) in &changed {
        for child in children {
            println!("changed, animation: {animation:?}");
            if let Some((_, mut player, mut transitions)) = players.iter_mut().find(|(p, _, _)| ***p == *child) {
                println!("starting, animation: {animation:?}, id: {child:?}");
                transitions
                    .play(&mut player, animation.get_node(&animations), Duration::from_millis(250))
                    .repeat();
            }
        }
    }
}
