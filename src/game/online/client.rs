use bevy::prelude::*;
use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

use crate::{game::base::resources::WorldScene, AppState};

use super::events::ReceivedWorld;

pub fn load_world(
    mut world_event: EventReader<ReceivedWorld>,
    type_registry: Res<AppTypeRegistry>,
    mut scenes: ResMut<Assets<DynamicScene>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let world_event = world_event.read().next().expect("shit");
    let mut deserializer = ron::de::Deserializer::from_str(&world_event.0).unwrap();
    let dynamic_scene = SceneDeserializer{type_registry: &type_registry.read()}.deserialize(&mut deserializer).unwrap();
    let dynamic_scene_handle = scenes.add(dynamic_scene);
    commands.insert_resource(WorldScene(dynamic_scene_handle));
    next_state.set(AppState::InGame);
}
