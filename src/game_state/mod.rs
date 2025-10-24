mod components;

use bevy::prelude::*;

pub use components::*;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, _app: &mut App) {}
}
