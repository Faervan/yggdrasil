use std::time::Instant;

use bevy::prelude::*;

use crate::{commands::{GameCommand, SettingToggle}, AppState};

use super::{components::{Follow, GameComponentParent, GlobalUiPosition}, resources::{GameAge, TimeInGame}};

pub fn insert_in_game_time(
    mut commands: Commands,
) {
    commands.insert_resource(TimeInGame(Time::new(Instant::now())));
}

pub fn insert_game_age(
    mut commands: Commands,
) {
    commands.insert_resource(GameAge::default());
}

pub fn advance_time(
    mut in_game_time: ResMut<TimeInGame>,
    mut game_age: ResMut<GameAge>,
    time: Res<Time>,
) {
    let delta = time.delta();
    in_game_time.0.advance_by(delta);
    game_age.time.advance_by(delta);
}

// Definetely not stolen from dubble https://discord.com/channels/691052431525675048/1204744148041732117/1205101881454633001
pub fn compute_screen_positions(
    query_camera: Query<(&GlobalTransform, &bevy::prelude::Camera)>,
    mut query: Query<(&mut GlobalUiPosition, &GlobalTransform), Without<Camera>>
) {
    let (global_camera_transform, camera) = query_camera.single();
    query.par_iter_mut().for_each(|(mut global_ui_pos, global_transform)| {
        if let Some(xy) = camera.world_to_viewport(
            global_camera_transform,
            global_transform.translation()
        ) {
            global_ui_pos.pos = xy;
        }
    });
}

// Definetely not stolen from dubble https://discord.com/channels/691052431525675048/1204744148041732117/1205101881454633001
pub fn follow_for_node(
    mut query_follow: Query<(&Follow, &Node, &mut Style)>,
    query_target: Query<(Entity, &GlobalUiPosition)>
) {
    for (follow, node, mut style) in query_follow.iter_mut() {
        if let Ok((_, target_pos)) = query_target.get(follow.entity) {
            let node_half_size = node.size() / 2.0;
            let target = target_pos.pos - node_half_size;
            style.margin.left = Val::Px(target.x);
            style.margin.top = Val::Px(target.y-150.);
        }
    }
}

pub fn despawn_all_entities(
    mut commands: Commands,
    entities: Query<Entity, With<GameComponentParent>>,
) {
    println!("full despawn");
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<TimeInGame>();
    commands.remove_resource::<GameAge>();
}

pub fn return_to_menu(
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

pub fn toggle_debug(
    mut event_writer: EventWriter<GameCommand>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F3) {
        event_writer.send(GameCommand::Toggle(SettingToggle::Debug));
    } else if (input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::AltLeft)) && input.just_pressed(KeyCode::KeyB){
        event_writer.send(GameCommand::Toggle(SettingToggle::Hitboxes));
    }
}
