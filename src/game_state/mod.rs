mod components;
mod systems;

use bevy::input::common_conditions::*;
use bevy::prelude::*;

pub use components::*;

use crate::core::GGConfig;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoverCell {
            cell: None,
            world_hit: None,
        });

        app.insert_resource(VisualizePolicy(false));

        app.add_systems(
            Startup,
            (systems::setup_hover_box, systems::thicker_gizmos)
                .run_if(|config: Res<GGConfig>| !config.headless),
        );

        app.add_systems(
            Update,
            (
                systems::update_hover_box,
                systems::cursor_to_grid_cell,
                systems::visualize_policy,
                systems::toggle_policy_visualization.run_if(input_just_pressed(KeyCode::KeyP)),
            )
                .run_if(|config: Res<GGConfig>| !config.headless),
        );
    }
}
