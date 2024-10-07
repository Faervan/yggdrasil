use bevy::{audio::Volume, prelude::*};

use crate::Settings;

pub struct SoundPlugin;
#[derive(Component)]
pub struct Music;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, play_music)
            .add_systems(Update, play_sfx);
    }
}

fn play_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("embedded://sounds/1006.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: !settings.music_enabled,
                volume: Volume::new(0.3),
                ..default()
            },
        },
        Music {},
    ));
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
                        source: asset_server.load("embedded://sounds/Eff2.ogg"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: Volume::new(0.2),
                            ..default()
                        }
                    });
                }
                Interaction::Pressed => {
                    commands.spawn(AudioBundle {
                        source: asset_server.load("embedded://sounds/Eff3.ogg"),
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
