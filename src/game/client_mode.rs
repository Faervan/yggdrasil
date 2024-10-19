use bevy::prelude::*;
use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

use crate::{AppState, ReceivedWorld};

pub fn load_world(
    mut world_event: EventReader<ReceivedWorld>,
    type_registry: Res<AppTypeRegistry>,
    mut scenes: ResMut<Assets<DynamicScene>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let world_event = world_event.read().next().expect("shit");
    println!("received world event:\nstart\n{}\ndone", world_event.0);
    let mut deserializer = ron::de::Deserializer::from_str(&world_event.0).unwrap();
    let dynamic_scene = SceneDeserializer{type_registry: &type_registry.read()}.deserialize(&mut deserializer).unwrap();
    let dynamic_scene_handle = scenes.add(dynamic_scene);
    commands.spawn(DynamicSceneBundle {
        scene: dynamic_scene_handle,
        ..default()
    });
    next_state.set(AppState::InGame);
}
