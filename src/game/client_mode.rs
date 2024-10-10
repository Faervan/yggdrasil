use bevy::prelude::*;
use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

use crate::{AppState, ReceivedWorld};

pub fn load_world(
    //mut world_event: EventReader<ReceivedWorld>,
    type_registry: Res<AppTypeRegistry>,
    mut scenes: ResMut<Assets<DynamicScene>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    //let world_event = world_event.read().next().expect("shit");
    let string = r#"(
  resources: {},
  entities: {
    8589934593: (
      components: {
        "bevy_transform::components::transform::Transform": (
          translation: (
            x: -0.1215124,
            y: 3.9994566,
            z: -0.121507265,
          ),
          rotation: (
            x: 0.0,
            y: -0.7028805,
            z: 0.0,
            w: 0.71130794,
          ),
          scale: (
            x: 0.4,
            y: 0.4,
            z: 0.4,
          ),
        ),
        "yggdrasil::game::components::Player": (
          base_velocity: 10.0,
          name: "Jon",
        ),
        "yggdrasil::game::components::Health": (
          value: 5,
        ),
      },
    ),
    8589934598: (
      components: {
        "bevy_transform::components::transform::Transform": (
          translation: (
            x: 30.0,
            y: 3.9994566,
            z: 0.0,
          ),
          rotation: (
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
          ),
          scale: (
            x: 0.4,
            y: 0.4,
            z: 0.4,
          ),
        ),
        "yggdrasil::game::components::Health": (
          value: 4,
        ),
        "yggdrasil::game::components::Npc": (),
      },
    ),
  },
)"#.to_string();
    println!("{}", string);
    let mut deserializer = ron::de::Deserializer::from_str(&string).unwrap();
    let dynamic_scene = SceneDeserializer{type_registry: &type_registry.read()}.deserialize(&mut deserializer).unwrap();
    let dynamic_scene_handle = scenes.add(dynamic_scene);
    commands.spawn(DynamicSceneBundle {
        scene: dynamic_scene_handle,
        ..default()
    });
    next_state.set(AppState::InGame);
}
