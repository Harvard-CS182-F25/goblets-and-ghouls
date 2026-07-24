mod components;
mod systems;

use bevy::input::common_conditions::*;
use bevy::prelude::*;

pub use components::*;

use crate::resources::{ConfigResource, HeatmapResource};

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoverCell {
            cell: None,
            world_hit: None,
        });

        app.insert_resource(VisualizePolicy(false));
        app.insert_resource(VisualizeValue(false));
        app.insert_resource(VisualizeVisibility(false));
        app.insert_resource(HeatmapColorRange::default());

        app.add_systems(
            Startup,
            (
                systems::setup_hover_box,
                systems::thicker_gizmos,
                systems::spawn_heatmap_tiles,
                systems::spawn_visibility_tiles,
            )
                .run_if(|config: Res<ConfigResource>| !config.0.headless),
        );

        app.add_systems(
            Update,
            (
                systems::update_hover_box,
                systems::cursor_to_grid_cell,
                systems::visualize_policy,
                systems::toggle_policy_visualization.run_if(input_just_pressed(KeyCode::KeyP)),
                systems::update_heatmap_color_range.run_if(resource_changed::<HeatmapResource>),
                systems::update_heatmap,
                systems::toggle_value_visualization.run_if(input_just_pressed(KeyCode::KeyV)),
                systems::update_visibility_overlay,
                systems::toggle_visibility_overlay.run_if(input_just_pressed(KeyCode::KeyF)),
                systems::show_game_over_overlay,
            )
                .run_if(|config: Res<ConfigResource>| !config.0.headless),
        );
    }
}
