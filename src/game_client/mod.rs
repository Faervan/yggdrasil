use bevy::prelude::*;
use crossbeam::channel::{Sender, Receiver, unbounded};
use std::{io, thread};
use std::net::UdpSocket;

use crate::{AppState, GameSessionType};
use crate::ui::{NORMAL_BUTTON, PRESSED_BUTTON, HOVERED_BUTTON, Camera2d};

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        let (msg_sender, msg_receiver) = unbounded::<String>();
        let (con_end_sender, con_end_receiver) = unbounded::<bool>();
        app
            .add_systems(OnEnter(AppState::InGame(GameSessionType::GameClient)), (
                    spawn_camera,
                    build_ui,
                    connect_socket,
                ))
            .add_systems(Update, (
                    update_ui,
                    menu_interaction,
                ).run_if(in_state(AppState::InGame(GameSessionType::GameClient))))
            .add_systems(OnExit(AppState::InGame(GameSessionType::GameClient)), (
                    despawn_camera,
                    despawn_ui,
                    disconnect_socket,
                ))
            .insert_resource(SocketThreadChannel {
                s: con_end_sender,
                r: con_end_receiver
            })
            .insert_resource(HostChannel {
                s: msg_sender,
                r: msg_receiver
            });
    }
}

#[derive(Resource)]
struct SocketThreadChannel {
    s: Sender<bool>,
    r: Receiver<bool>
}

#[derive(Resource)]
struct HostChannel {
    s: Sender<String>,
    r: Receiver<String>
}

#[derive(Resource)]
struct MessageBox {
    box_entity: Entity,
}

#[derive(Component)]
struct MsgReceiveText;

#[derive(Component)]
struct MainMenuButton;

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
            //Received msg text
            parent
                .spawn((
                    TextBundle {
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
                        text: Text::from_section(
                            "Receiving...",
                            TextStyle {
                                font_size: 33.0,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ),
                        ..default()
                    },
                    MsgReceiveText {},
                ));
            // Back to main menu button
            parent
                .spawn((
                    ButtonBundle {
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
                    },
                    MainMenuButton {}
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back to Main Menu",
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
    mut message_box: Query<&mut Text, With<MsgReceiveText>>,
    host_channel: Res<HostChannel>,
) {
    if let Ok(mut text) = message_box.get_single_mut() {
        if let Ok(content) = host_channel.r.try_recv() {
            text.sections[0].value = content;
        }
    }
}

fn menu_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut main_menu_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<MainMenuButton>)>,
) {
    for (interaction, mut color) in &mut main_menu_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::MainMenu);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn connect_socket(
    socket_thread_channel: Res<SocketThreadChannel>,
    host_channel: Res<HostChannel>,
) {
    let socket_thread_channel_receiver = socket_thread_channel.r.clone();
    let host_channel_sender = host_channel.s.clone();
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:9984").expect("Failed to bind to address");
        socket.set_nonblocking(true).unwrap();
        socket.send_to("connect".as_bytes(), "91.108.102.51:9983").expect("Couldn't connect to server");
        let mut buf = [0; 20];
        'outer: loop {
            let _ = loop {
                if socket_thread_channel_receiver.try_recv() == Ok(true) {
                    break 'outer
                }
                match socket.recv_from(&mut buf) {
                    Ok(x) => break x,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {continue;},
                    Err(e) => { panic!("Encountered io error: {e}") }
                }
            };
            host_channel_sender.send(String::from_utf8_lossy(&buf).to_string()).unwrap();
            buf = [0; 20];
        }
    });
}

fn disconnect_socket(
    sender: Res<SocketThreadChannel>,
) {
    sender.s.send(true).unwrap();
}
