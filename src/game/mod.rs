use base::GameBasePlugin;
use bevy::prelude::*;

pub mod base;
pub mod online;
pub mod hud;

use hud::HudPlugin;
use online::GameOnlinePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                HudPlugin,
                GameBasePlugin,
                GameOnlinePlugin,
            ));
    }
}
