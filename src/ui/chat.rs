use bevy::{input::{keyboard::{Key, KeyboardInput}, ButtonState}, prelude::*};

use super::{NORMAL_BUTTON, HOVERED_BUTTON};

pub struct ChatPlugin;

#[derive(States, Default, Hash, Eq, PartialEq, Clone, Debug)]
pub enum ChatState {
    Open,
    Closed,
    #[default]
    Unpresent,
}

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(ChatState::Open), build_chat)
            .add_systems(OnEnter(ChatState::Closed), despawn_chat)
            .add_systems(Update, toggle_chat)
            .add_systems(Update, (
                get_chat_input,
            ).run_if(in_state(ChatState::Open)))
            .init_state::<ChatState>();
    }
}

#[derive(Component)]
struct ChatBox;

#[derive(Component)]
struct ChatMessages;

#[derive(Component)]
struct ChatInput(String);

fn build_chat(
    mut commands: Commands,
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
            ChatMessages {},
        ));
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
    chat_messages: Query<Entity, With<ChatMessages>>,
) {
    for event in events.read() {
        if event.state == ButtonState::Released {
            continue;
        }
        let (mut text, mut buffer) = chat_input.single_mut();

        match &event.logical_key {
            Key::Enter => {
                commands.spawn(
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
                            buffer.0.to_string(),
                            TextStyle {
                                font_size: 20.,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            }
                        )
                    );
                }).set_parent(chat_messages.get_single().unwrap());
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
