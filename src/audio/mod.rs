use bevy::{audio::Volume, prelude::*};

pub struct SoundPlugin;

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
) {
    commands.spawn(AudioBundle {
        source: asset_server.load("embedded://sounds/1006.ogg"),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::new(0.3),
            ..default()
        }
    });
}

fn play_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
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
