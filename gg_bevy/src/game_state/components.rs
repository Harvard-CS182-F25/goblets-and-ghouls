use bevy::prelude::*;

#[derive(Component)]
pub struct HoverBox;

#[derive(Component)]
pub struct HoverBoxText;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct HoverCell {
    pub cell: Option<UVec2>, // (col, row)
    pub world_hit: Option<Vec3>,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct VisualizePolicy(pub bool);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct VisualizeValue(pub bool);

/// Marks a persistent per-cell heatmap tile entity and the agent cell it
/// represents.
#[derive(Component, Clone, Copy)]
pub struct HeatmapTile(pub (usize, usize));

/// Cached global min/max of the current `HeatmapResource`'s values, used to
/// normalize tile colors stably rather than renormalizing every frame.
/// Recomputed only when `HeatmapResource` changes.
#[derive(Resource, Default)]
pub struct HeatmapColorRange(pub Option<(f32, f32)>);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct VisualizeVisibility(pub bool);

/// Marks a persistent per-cell visibility-overlay tile entity and the cell
/// it represents. Floor cells sit over the floor, wall cells over the wall
/// tops. Darkened (translucent black) whenever that cell is not currently
/// visible from the agent's position; hidden otherwise.
#[derive(Component, Clone, Copy)]
pub struct VisibilityTile(pub (usize, usize));

/// Marks the full-screen game-over overlay (see `show_game_over_overlay`),
/// so it's spawned at most once and can be torn down again if the episode
/// ever resets.
#[derive(Component)]
pub struct GameOverOverlay;
