use bevy::prelude::*;

use crate::scene::WALL_HEIGHT;

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Wall;

#[derive(Debug, Clone, Bundle, Default)]
pub struct WallBundle {
    pub wall: Wall,
    pub position: Transform,
}

impl WallBundle {
    pub fn new(endpoint1: Vec2, endpoint2: Vec2) -> Self {
        let diff = endpoint2 - endpoint1;
        let center = (endpoint1 + endpoint2) * 0.5;
        let yaw = diff.y.atan2(diff.x);

        let position =
            Transform::from_translation(Vec3::new(center.x, WALL_HEIGHT / 2.0, center.y))
                .with_rotation(Quat::from_rotation_y(yaw));

        Self {
            wall: Wall,
            position,
        }
    }
}
