use std::time::Duration;

use bevy::{prelude::*, reflect::DynamicTupleStruct};
use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

use crate::{game::base::resources::{GameAge, WorldScene}, AppState};

use super::{events::ReceivedWorld, resource::GameAgeDuration};

pub fn load_world(
    mut world_event: EventReader<ReceivedWorld>,
    type_registry: Res<AppTypeRegistry>,
    mut scenes: ResMut<Assets<DynamicScene>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let world_event = world_event.read().next().expect("shit");
    let mut deserializer = ron::de::Deserializer::from_str(&world_event.0).unwrap();
    let mut dynamic_scene = SceneDeserializer{type_registry: &type_registry.read()}.deserialize(&mut deserializer).unwrap();
    dynamic_scene.resources.iter()
        .position(|r| r.represents::<GameAgeDuration>())
        .map(|pos| {
            let age = *dynamic_scene.resources
                .swap_remove(pos)
                .downcast_ref::<DynamicTupleStruct>().unwrap()
                .field(0).unwrap()
                .downcast_ref::<Duration>().unwrap();
            dynamic_scene.resources.push(
                Box::new(GameAge::from_duration(age))
            );
        });
    let dynamic_scene_handle = scenes.add(dynamic_scene);
    commands.insert_resource(WorldScene(dynamic_scene_handle));
    next_state.set(AppState::InGame);
}
