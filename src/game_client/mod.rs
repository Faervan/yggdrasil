use bevy::prelude::*;
use crossbeam::channel::{Receiver, unbounded};
use std::thread;

use crate::{AppState, GameSessionType};
use crate::ui::{NORMAL_BUTTON, Camera2d};

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = unbounded::<String>();
        thread::spawn(move ||
            loop {
                sender.send("".to_string()).unwrap();
            }
        );
        app
            .add_systems(OnEnter(AppState::InGame(GameSessionType::GameClient)), (
                    spawn_camera,
                    build_ui,
                ))
            .add_systems(Update, (
                    update_ui,
                ).run_if(in_state(AppState::InGame(GameSessionType::GameClient))))
            .add_systems(OnExit(AppState::InGame(GameSessionType::GameClient)), (
                    despawn_camera,
                    despawn_ui,
                ))
            .insert_resource(MsgReceiver(receiver));
    }
}

#[derive(Resource)]
struct MsgReceiver(Receiver<String>);

#[derive(Resource)]
struct MessageBox {
    box_entity: Entity,
}

pub fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((Camera2dBundle::default(), Camera2d {}));
}

fn despawn_camera(
    mut commands: Commands,
    camera: Query<Entity, With<Camera2d>>,
) {
    commands.entity(camera.get_single().unwrap()).despawn();
}

fn build_ui(
    mut commands: Commands,
) {
    let box_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Receiving...",
                        TextStyle {
                            font_size: 33.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        }).id();
    commands.insert_resource(MessageBox { box_entity });
}

fn despawn_ui(
    message_box: Res<MessageBox>,
    mut commands: Commands,
) {
    commands.entity(message_box.box_entity).despawn_recursive();
}

fn update_ui(
    mut message_box: Query<&mut Text>,
) {
    if let Ok(mut text) = message_box.get_single_mut() {
        text.sections[0].value = "New value".to_string();
    }
}
