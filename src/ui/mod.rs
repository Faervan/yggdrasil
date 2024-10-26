use bevy::prelude::*;

use crate::AppState;

use self::{chat::ChatPlugin, helper::TextfieldPlugin, lobby::LobbyPlugin};

pub mod chat;
pub mod lobby;
pub mod helper;
mod components;

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                ChatPlugin {},
                TextfieldPlugin {},
                LobbyPlugin {},
            ))
            .insert_resource(MenuData {entities: vec![]})
            .add_systems(OnEnter(AppState::MainMenu), (
                spawn_camera,
                build_main_menu,
            ))
            .add_systems(Update, menu_interaction)
            .add_systems(OnExit(AppState::MainMenu), (
                despawn_camera,
                despawn_menu,
            ));
    }
}

#[derive(Component)]
pub struct Camera2d;

#[derive(Resource)]
struct MenuData {
    entities: Vec<Entity>,
}

fn spawn_camera(
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

#[derive(Component)]
struct SinglePlayerButton;

#[derive(Component)]
struct MultiPlayerButton;

#[derive(Component)]
struct AppExitButton;

fn build_main_menu(
    mut commands: Commands,
    mut menudata: ResMut<MenuData>,
) {
    let entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Button for Singleplayer
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(250.),
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
                    SinglePlayerButton {}
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 33.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
            // Button for multiplayer client
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(250.),
                            height: Val::Px(65.),
                            margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(5.), Val::ZERO),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    MultiPlayerButton {}
                ))
                .with_children(|parent| {
                    let style = TextStyle {
                        font_size: 33.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    };
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new("Play ", style.clone()),
                        TextSection::new("online", style.clone()),
                    ]));
                });
            // App exit Button
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(250.),
                            height: Val::Px(65.),
                            margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(5.), Val::ZERO),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    AppExitButton {}
                ))
                .with_children(|parent| {
                    let style = TextStyle {
                        font_size: 33.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    };
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new("Quit", style.clone()),
                    ]));
                });
        }).id();
    menudata.entities = vec![entity];
}

fn menu_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_event: EventWriter<AppExit>,
    mut singleplayer_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<SinglePlayerButton>)>,
    mut multiplayer_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<MultiPlayerButton>, Without<SinglePlayerButton>)>,
    mut app_exit_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<AppExitButton>, Without<SinglePlayerButton>, Without<MultiPlayerButton>)>,
) {
    for (interaction, mut color) in &mut singleplayer_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
    for (interaction, mut color) in &mut multiplayer_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::Lobby(crate::LobbyState::ConMenu));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
    for (interaction, mut color) in &mut app_exit_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                app_exit_event.send(AppExit::Success);
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

fn despawn_menu(
    menu_data: Res<MenuData>,
    mut commands: Commands,
) {
    for entity in menu_data.entities.clone() {
        commands.entity(entity).despawn_recursive();
    }
}
