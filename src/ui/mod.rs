use bevy::prelude::*;

use crate::AppState;

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::MainMenu), (
                spawn_camera,
                build_main_menu,
            ))
            .add_systems(Update, menu_interaction)
            .add_systems(OnExit(AppState::MainMenu), (
                despawn_camera.before(crate::game_client::spawn_camera),
                despawn_main_menu,
            ));
    }
}

#[derive(Component)]
pub struct Camera2d;

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
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

fn build_main_menu(
    mut commands: Commands,
) {
    let button_entity = commands
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
            // Button for Singleplayer
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
                    MultiPlayerButton {}
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play online",
                        TextStyle {
                            font_size: 33.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        }).id();
    commands.insert_resource(MenuData { button_entity });
}

fn menu_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut singleplayer_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<SinglePlayerButton>)>,
    mut multiplayer_interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<MultiPlayerButton>, Without<SinglePlayerButton>)>,
) {
    for (interaction, mut color) in &mut singleplayer_interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::InGame(crate::GameSessionType::Singleplayer));
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
                next_state.set(AppState::InGame(crate::GameSessionType::GameClient));
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

fn despawn_main_menu(
    menu_data: Res<MenuData>,
    mut commands: Commands,
) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}