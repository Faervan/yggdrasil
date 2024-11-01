use bevy::prelude::*;
use ysync::TcpFromClient;

use crate::{game::{base::{components::{Health, Npc, Player}, resources::GameAge}, online::resource::GameAgeDuration}, ui::lobby::LobbySocket};

pub fn share_world(
    world: &World,
    players: Query<Entity, With<Player>>,
    npcs: Query<Entity, With<Npc>>,
    remote: Res<LobbySocket>,
) {
    println!("executing share_world");
    let game_age = world.get_resource::<GameAge>().unwrap_or(&GameAge::default()).startup;
    let mut scene = DynamicSceneBuilder::from_world(world)
        .allow::<Player>()
        .allow::<Transform>()
        .allow::<GlobalTransform>()
        .allow::<Health>()
        .allow::<Npc>()
        .allow_resource::<GameAgeDuration>()
        .extract_entities(
            players.iter()
            .chain(npcs.iter())
        )
        .build();
    scene.resources.push(Box::new(GameAgeDuration(game_age.elapsed())));
    let serialized_scene = scene.serialize(&world.resource::<AppTypeRegistry>().read()).unwrap();
    let _ = remote.socket.tcp_send.send(TcpFromClient::GameWorld(serialized_scene));
}
