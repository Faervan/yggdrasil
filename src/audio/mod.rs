use bevy::{audio::Volume, prelude::*};

use crate::{AppState, Settings};

pub struct SoundPlugin;
#[derive(Component)]
pub struct Music;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, load_music)
            .add_systems(OnEnter(AppState::InGame), play_game_music)
            .add_systems(OnExit(AppState::InGame), play_lobby_music)
            .add_systems(Update, (
                play_sfx,
                music_fade_in,
                music_fade_out,
            ));
    }
}

#[derive(Resource)]
struct MusicHandles {
    lobby: Handle<AudioSource>,
    game: Handle<AudioSource>,
    active: Entity
}

#[derive(Component)]
struct FadeIn;
#[derive(Component)]
struct FadeOut;

fn load_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
) {
    let entity = commands.spawn((
        AudioBundle {
            source: asset_server.load("embedded://sounds/soundtrack_lobby.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: !settings.music_enabled,
                volume: Volume::new(0.3),
                ..default()
            },
        },
        Music {},
    )).id();
    commands.insert_resource(MusicHandles {
        lobby: asset_server.load("embedded://sounds/soundtrack_lobby.ogg"),
        game: asset_server.load("embedded://sounds/sirius_ingame.ogg"),
        active: entity,
    });
}

fn play_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    settings: Res<Settings>,
) {
    if settings.sfx_enabled {
        for interaction in buttons.iter() {
            match interaction {
                Interaction::Hovered => {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("embedded://sounds/eff2.ogg"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: Volume::new(0.2),
                            ..default()
                        }
                    });
                }
                Interaction::Pressed => {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("embedded://sounds/eff3.ogg"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: Volume::new(0.2),
                            ..default()
                        }
                    });
                }
                Interaction::None => {}
            }
        }
    }
}

fn play_game_music(
    mut commands: Commands,
    mut music: ResMut<MusicHandles>,
    settings: Res<Settings>,
) {
    commands.entity(music.active).insert(FadeOut);
    let entity = commands.spawn((
        AudioBundle {
            source: music.game.clone_weak(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: !settings.music_enabled,
                volume: Volume::new(0.3),
                ..default()
            },
        },
        Music {},
    )).id();
    music.active = entity;
}

fn play_lobby_music(
    mut commands: Commands,
    mut music: ResMut<MusicHandles>,
    settings: Res<Settings>,
) {
    commands.entity(music.active).insert(FadeOut);
    let entity = commands.spawn((
        AudioBundle {
            source: music.lobby.clone_weak(),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: !settings.music_enabled,
                volume: Volume::new(0.3),
                ..default()
            },
        },
        Music {},
    )).id();
    music.active = entity;
}

const FADE_TIME: f32 = 2.;

fn music_fade_in(
    mut commands: Commands,
    mut audio: Query<(&mut AudioSink, Entity), With<FadeIn>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio.iter_mut() {
        audio.set_volume(audio.volume() + time.delta_seconds() / FADE_TIME);
        if audio.volume() >= 1.0 {
            audio.set_volume(1.0);
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

fn music_fade_out(
    mut commands: Commands,
    mut audio: Query<(&mut AudioSink, Entity), With<FadeOut>>,
    time: Res<Time>,
) {
    for (audio, entity) in audio.iter_mut() {
        audio.set_volume(audio.volume() - time.delta_seconds() / FADE_TIME);
        if audio.volume() <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
