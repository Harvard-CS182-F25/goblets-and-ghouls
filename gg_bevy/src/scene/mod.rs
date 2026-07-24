mod components;
mod systems;
mod visual;

use bevy::prelude::*;

pub use components::*;
pub use visual::*;

use crate::core::StartupSets;
use crate::resources::ConfigResource;

pub const WALL_HEIGHT: f32 = 5.0;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1_500.0,
            ..Default::default()
        });

        app.add_systems(PreStartup, init_wall_assets);
        app.add_systems(
            Startup,
            (
                (systems::setup_scene, systems::spawn_walls).in_set(StartupSets::Walls),
                systems::setup_key_instructions,
            ),
        );
        app.add_systems(
            Update,
            systems::update_reward_text.run_if(|config: Res<ConfigResource>| !config.0.headless),
        );
    }
}

fn init_wall_assets(mut commands: Commands, config: Res<ConfigResource>) {
    if !config.0.headless {
        commands.init_resource::<WallGraphicsAssets>();
    }
}
