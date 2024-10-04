use std::collections::VecDeque;

use bevy::{input::{keyboard::{Key, KeyboardInput}, ButtonState}, prelude::*};

use super::{lobby::{disconnet_from_lobby, send_msg_to_lobby, ConnectionState}, HOVERED_BUTTON, NORMAL_BUTTON};

pub struct ChatPlugin;

#[derive(States, Default, Hash, Eq, PartialEq, Clone, Debug)]
pub enum ChatState {
    Open,
    Closed,
    #[default]
    Unpresent,
}

#[derive(Resource)]
struct ChatMessages(VecDeque<(Entity, String)>);
#[derive(Resource)]
pub struct PendingMessages(pub Vec<String>);

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ChatMessages(VecDeque::new()))
            .insert_resource(PendingMessages(Vec::new()))
            .add_systems(OnEnter(ChatState::Open), build_chat)
            .add_systems(OnEnter(ChatState::Closed), despawn_chat)
            .add_systems(Update, toggle_chat)
            .add_systems(Update, (
                get_chat_input,
                spawn_pending_messages,
            ).run_if(in_state(ChatState::Open)))
            .add_systems(Update, send_msg_to_lobby.before(disconnet_from_lobby).run_if(in_state(ChatState::Open)).run_if(in_state(ConnectionState::Connected)).before(get_chat_input))
            .add_systems(Update, dump_pending_messages.run_if(in_state(ChatState::Closed)))
            .init_state::<ChatState>();
    }
}

#[derive(Component)]
struct ChatBox;

#[derive(Component)]
struct ChatMessageBox;

#[derive(Component)]
pub struct ChatInput(pub String);

fn build_chat(
    mut commands: Commands,
    mut chat_messages: ResMut<ChatMessages>,
) {
    // Chat box
    commands.spawn((
        NodeBundle {
            style: Style {
                // center button
                width: Val::Px(700.),
                height: Val::Px(400.),
                justify_content: JustifyContent::End,
                align_items: AlignItems::End,
                align_self: AlignSelf::End,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: NORMAL_BUTTON.with_alpha(0.7).into(),
            ..default()
        },
        ChatBox {},
    )).with_children(|p| {
        p.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Px(360.),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::End,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    ..default()
                },
                ..default()
            },
            ChatMessageBox {},
        )).with_children(|p| {
            for (id, msg) in chat_messages.0.iter_mut() {
                *id = p.spawn(
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            padding: UiRect::px(10., 10., 0., 0.),
                            ..default()
                        },
                        ..default()
                    }
                ).with_children(|p| {
                    p.spawn(
                        TextBundle::from_section(
                            msg.clone(),
                            TextStyle {
                                font_size: 20.,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            }
                        )
                    );
                }).id();
            }
        });
        p.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                min_height: Val::Px(40.),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                padding: UiRect::px(10., 10., 10., 10.),
                ..default()
            },
            background_color: HOVERED_BUTTON.with_alpha(0.7).into(),
            ..default()
        }).with_children(|p| {
            p.spawn((
                TextBundle::from_section(
                    "Start typing ...",
                    TextStyle {
                        font_size: 20.,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    }
                ),
                ChatInput("".to_string()),
            ));
        });
    });
}

fn despawn_chat(
    chat: Query<Entity, With<ChatBox>>,
    mut commands: Commands,
) {
    commands.entity(chat.get_single().unwrap()).despawn_recursive();
}

fn toggle_chat(
    input: Res<ButtonInput<KeyCode>>,
    chat_state: Res<State<ChatState>>,
    mut next_state: ResMut<NextState<ChatState>>,
) {
    if *chat_state.get() == ChatState::Open {
        if input.just_pressed(KeyCode::Escape) {
            next_state.set(ChatState::Closed);
        }
    } else {
        if input.just_pressed(KeyCode::Enter) {
            next_state.set(ChatState::Open);
        }
    }
}

fn get_chat_input(
    mut events: EventReader<KeyboardInput>,
    mut chat_input: Query<(&mut Text, &mut ChatInput)>,
    mut commands: Commands,
    chat_message_box: Query<Entity, With<ChatMessageBox>>,
    mut chat_messages: ResMut<ChatMessages>,
) {
    for event in events.read() {
        if event.state == ButtonState::Released {
            continue;
        }
        let (mut text, mut buffer) = chat_input.single_mut();

        match &event.logical_key {
            Key::Enter => {
                let msg_content = buffer.0.to_string();
                let msg_id = commands.spawn(
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            justify_content: JustifyContent::Start,
                            align_items: AlignItems::Center,
                            padding: UiRect::px(10., 10., 0., 0.),
                            ..default()
                        },
                        ..default()
                    }
                ).with_children(|p| {
                    p.spawn(
                        TextBundle::from_section(
                            msg_content.clone(),
                            TextStyle {
                                font_size: 20.,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            }
                        )
                    );
                }).set_parent(chat_message_box.get_single().unwrap()).id();
                chat_messages.0.push_back((msg_id, msg_content));
                if chat_messages.0.len() > 50 {
                    let (overflow, _) = chat_messages.0.pop_front().unwrap();
                    commands.entity(overflow).despawn();
                }
                buffer.0 = "".to_string();
                text.sections[0].value = "Start typing ...".to_string();
            }
            Key::Space => {
                buffer.0.push(' ');
                text.sections[0].value = buffer.0.to_string();
            }
            Key::Backspace => {
                buffer.0.pop();
                text.sections[0].value = buffer.0.to_string();
            }
            Key::Character(character) => {
                buffer.0.push_str(character);
                text.sections[0].value = buffer.0.to_string();
            }
            _ => continue,
        }
    }
}

fn spawn_pending_messages(
    mut pending_messages: ResMut<PendingMessages>,
    mut chat_messages: ResMut<ChatMessages>,
    chat_message_box: Query<Entity, With<ChatMessageBox>>,
    mut commands: Commands,
    ) {
    for msg in pending_messages.0.iter_mut() {
        let id = commands.spawn(
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    padding: UiRect::px(10., 10., 0., 0.),
                    ..default()
                },
                ..default()
            }
        ).with_children(|p| {
            p.spawn(
                TextBundle::from_section(
                    msg.clone(),
                    TextStyle {
                        font_size: 20.,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    }
                )
            );
        }).set_parent(chat_message_box.get_single().unwrap()).id();
        chat_messages.0.push_back((id, msg.to_string()));
        if chat_messages.0.len() > 50 {
            let (overflow, _) = chat_messages.0.pop_front().unwrap();
            commands.entity(overflow).despawn();
        }
    }
    pending_messages.0 = Vec::new();
}

fn dump_pending_messages(
    mut pending_messages: ResMut<PendingMessages>,
    mut chat_messages: ResMut<ChatMessages>,
) {
    for msg in pending_messages.0.iter_mut() {
        chat_messages.0.push_back((Entity::from_raw(0), msg.to_string()));
    }
    pending_messages.0 = Vec::new();
}
