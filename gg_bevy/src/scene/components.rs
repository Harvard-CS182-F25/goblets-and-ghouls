use bevy::prelude::*;

use crate::scene::WALL_HEIGHT;

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Wall;

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct GroundPlane;

/// Marks the episode-seed line in the upper-right HUD panel so
/// `update_hud_text` can refresh it after a restart.
#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EpisodeSeedText;

/// Marks the reward line in the upper-right HUD panel so `update_hud_text`
/// can refresh it every frame.
#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct RewardText;

#[derive(Debug, Clone, Bundle, Default)]
pub struct WallBundle {
    pub wall: Wall,
    pub position: Transform,
}

impl WallBundle {
    pub fn new(center: Vec2) -> Self {
        let position =
            Transform::from_translation(Vec3::new(center.x, WALL_HEIGHT / 2.0, center.y));

        Self {
            wall: Wall,
            position,
        }
    }
}
