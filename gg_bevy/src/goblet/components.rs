use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FlagStatus {
    Dropped,
    PickedUp,
    Captured,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub struct Goblet {
    value: i32,
}

#[derive(Bundle)]
pub struct GobletBundle {
    pub name: Name,
    pub goblet: Goblet,
    pub transform: Transform,
}

impl GobletBundle {
    pub fn new(name: &str, position: Vec3, value: i32) -> Self {
        Self {
            name: Name::new(name.to_string()),
            goblet: Goblet { value },
            transform: Transform::from_translation(position),
        }
    }
}
