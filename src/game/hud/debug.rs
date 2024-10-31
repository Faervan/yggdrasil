use bevy::{core::FrameCount, prelude::*};

use crate::{game::base::resources::{GameAge, TimeInGame}, ui::lobby::LobbySocket, Settings};

use super::{FpsInfo, FpsInfoText, GameAgeInfoText, HudDebugState, HudParentEntities, InGameTimeInfoText, PingInfoText};

pub fn build_debug_hud(
    mut commands: Commands,
    mut hud_entities: ResMut<HudParentEntities>,
    frame_count: Res<FrameCount>,
) {
    commands.insert_resource(FpsInfo {timer: Timer::from_seconds(0.25, TimerMode::Repeating), last_fps: frame_count.0});
    let debug_hud_id = commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            ..default()
        },
        ..default()
    }).with_children(|p| {
        let text_style = TextStyle {font_size: 30., color: Color::BLACK, ..default()};
        p.spawn((
            TextBundle::from_section("Fps: 0", text_style.clone()),
            FpsInfoText
        ));
        p.spawn((
            TextBundle::from_section("Ping: 0ms", text_style.clone()),
            PingInfoText
        ));
        p.spawn((
            TextBundle::from_section("TimeInGame: 0s", text_style.clone()),
            InGameTimeInfoText
        ));
        p.spawn((
            TextBundle::from_section("Game age: 0s", text_style),
           GameAgeInfoText
        ));
    }).id();
    hud_entities.debug = Some(debug_hud_id);
}

pub fn despawn_debug_hud(
    mut commands: Commands,
    mut hud_entities: ResMut<HudParentEntities>,
) {
    if let Some(debug_hud_id) = hud_entities.debug {
        commands.entity(debug_hud_id).despawn_recursive();
        hud_entities.debug = None;
        commands.remove_resource::<FpsInfo>();
    }
}

pub fn update_fps(
    mut fps_text: Query<&mut Text, With<FpsInfoText>>,
    frame_count: Res<FrameCount>,
    mut fps_info: ResMut<FpsInfo>,
    time: Res<Time>,
) {
    fps_info.timer.tick(time.delta());
    if fps_info.timer.finished() {
        if let Ok(mut text) = fps_text.get_single_mut() {
            let new_count = frame_count.0;
            text.sections[0].value = format!("Fps: {}", new_count.wrapping_sub(fps_info.last_fps) * 4);
            fps_info.last_fps = new_count;
        }
    }
}

pub fn update_ping(
    mut info_text: Query<&mut Text, With<PingInfoText>>,
    mut remote: ResMut<LobbySocket>,
) {
    if remote.socket.ping.has_changed().is_ok() {
        if let Ok(mut text) = info_text.get_single_mut() {
            let ping = remote.socket.ping.borrow_and_update();
            text.sections[0].value = format!("Ping: {}ms", ping.as_millis());
        }
    }
}

pub fn update_in_game_time(
    mut info_text: Query<&mut Text, With<InGameTimeInfoText>>,
    game_time: Res<TimeInGame>,
) {
    if let Ok(mut text) = info_text.get_single_mut() {
        text.sections[0].value = format!("Time in game: {}s", (game_time.0.elapsed_seconds() * 10.).round() / 10.);
    }
}

pub fn update_game_age(
    mut info_text: Query<&mut Text, With<GameAgeInfoText>>,
    game_age: Res<GameAge>,
) {
    println!("game age exists");
    if let Ok(mut text) = info_text.get_single_mut() {
        text.sections[0].value = format!("Game age: {}s", (game_age.time.elapsed_seconds() * 10.).round() / 10.);
    }
}

pub fn try_set_debug_hud(
    settings: Res<Settings>,
    mut debug_hud_state: ResMut<NextState<HudDebugState>>,
) {
    if settings.debug_hud_enabled {
        debug_hud_state.set(HudDebugState::Enabled);
    }
}

pub fn try_remove_debug_hud(
    settings: Res<Settings>,
    mut debug_hud_state: ResMut<NextState<HudDebugState>>,
) {
    if settings.debug_hud_enabled {
        debug_hud_state.set(HudDebugState::Disabled);
    }
}
