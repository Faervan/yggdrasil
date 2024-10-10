use bevy::prelude::*;
use ysync::client::TcpPackage;

use crate::{game::components::*, ui::lobby::LobbySocket};

pub fn share_world(
    world: &World,
    players: Query<Entity, With<Player>>,
    npcs: Query<Entity, With<Npc>>,
    socket: Res<LobbySocket>,
) {
    println!("executing share_world");
    let scene = DynamicSceneBuilder::from_world(world)
        .allow::<Player>()
        .allow::<Transform>()
        .allow::<Health>()
        .allow::<Npc>()
        .extract_entities(
            players.iter()
            .chain(npcs.iter())
        ).build();
    let serialized_scene = scene.serialize(&world.resource::<AppTypeRegistry>().read()).unwrap();
    println!("{serialized_scene}");
    let _ = socket.socket.tcp_send.send(TcpPackage::GameWorld(serialized_scene));
}
