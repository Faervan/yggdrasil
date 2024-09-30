use bevy::{ecs::system::EntityCommands, input::{keyboard::{Key, KeyboardInput}, mouse::MouseButtonInput, ButtonState}, prelude::*};

use super::{HOVERED_BUTTON, NORMAL_BUTTON};

pub struct TextfieldPlugin;

impl Plugin for TextfieldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<TextfieldState>()
            .insert_resource(FocusedTextField(None))
            .add_systems(Update, textfield_focus.run_if(in_state(TextfieldState::Unfocused)))
            .add_systems(Update, (
                textfield_unfocus,
                get_text_input,
            ).run_if(in_state(TextfieldState::Focused)));
    }
}

#[derive(Default, States, PartialEq, Eq, Hash, Clone, Debug)]
enum TextfieldState {
    #[default]
    Unfocused,
    Focused
}

#[derive(Component)]
pub struct TextField;
#[derive(Component)]
pub struct TextFieldContent(pub String);
#[derive(Resource)]
pub struct FocusedTextField(Option<Entity>);

pub trait Textfield<'a> {
    fn as_textfield<T: Component>(self, placeholder: &str, accessor: T, width: Val, height: Val, font_size: f32) -> EntityCommands<'a>;
}
impl<'a> Textfield<'a> for EntityCommands<'a> {
    fn as_textfield<T: Component>(mut self, placeholder: &str, accessor: T, width: Val, height: Val, font_size: f32) -> EntityCommands<'a> {
        self.insert((    
            NodeBundle {
                style: Style {
                    width,
                    height,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    padding: UiRect::px(10., 10., 10., 10.),
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            Interaction::None,
            TextField {},
        ));
        self.with_children(|p| {
            p.spawn((
                TextBundle::from_section(
                    placeholder,
                    TextStyle {
                        font_size,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    }
                ),
                TextFieldContent("".to_string()),
                accessor,
            ));
        });
        self
    }
}

fn textfield_focus(
    mut interaction_query: Query<(&Interaction, Entity, &mut BackgroundColor), (Changed<Interaction>, With<TextField>)>,
    mut next_state: ResMut<NextState<TextfieldState>>,
) {
    for (interaction, entity, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
            Interaction::Pressed => {
                *color = NORMAL_BUTTON.into();
                next_state.set(TextfieldState::Focused);
            }
        }
    }
}

fn get_text_input(
    mut events: EventReader<KeyboardInput>,
    mut textfields: Query<(&mut Text, &mut TextFieldContent)>,
) {
    for event in events.read() {
        if event.state == ButtonState::Released {
            continue;
        }
        let (mut text, mut buffer) = textfields.single_mut();

        match &event.logical_key {
            Key::Enter => {
                println!("pressed enter!");
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

fn textfield_unfocus(
    click_event: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<TextfieldState>>,
    textfield_interaction: Query<&Interaction, (Changed<Interaction>, With<TextField>)>,
) {
    if click_event.just_pressed(MouseButton::Left) {
        for interaction in &textfield_interaction {
            if *interaction == Interaction::Pressed {
                return;
            }
        }
        next_state.set(TextfieldState::Unfocused);
    }
}
