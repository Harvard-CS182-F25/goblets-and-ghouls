mod components;
mod systems;
mod visual;

use bevy::prelude::*;

use crate::core::StartupSets;
use crate::resources::ConfigResource;

pub use components::*;
pub use visual::*;

pub struct GobletPlugin;
impl Plugin for GobletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_flag_and_capture_point_assets);
        app.add_systems(
            Startup,
            (systems::spawn_goblets).in_set(StartupSets::Goblets),
        );
        app.add_systems(Update, systems::update_goblet_reward_labels);
    }
}

fn init_flag_and_capture_point_assets(mut commands: Commands, config: Res<ConfigResource>) {
    if !config.0.headless {
        commands.init_resource::<visual::GobletGraphicsAssets>();
    }
}
