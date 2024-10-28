use bevy::{prelude::*, window::CursorGrabMode};

use crate::AppState;

use super::camera::CameraState;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<CursorGrabState>()
            .add_systems(OnEnter(CursorGrabState::Grabbed), grab_cursor)
            .add_systems(OnEnter(CursorGrabState::Free), release_cursor)
            .add_systems(OnEnter(AppState::InGame), grab_cursor.run_if(in_state(CameraState::Normal)))
            .add_systems(OnExit(AppState::InGame), release_cursor);
    }
}

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum CursorGrabState {
    Grabbed,
    #[default]
    Free
}

fn grab_cursor(mut window: Query<&mut Window>) {
    let mut window = window.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

fn release_cursor(mut window: Query<&mut Window>) {
    let mut window = window.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
}
