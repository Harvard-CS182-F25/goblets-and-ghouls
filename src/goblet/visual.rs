use bevy::prelude::*;

#[derive(Resource)]
pub struct GobletGraphicsAssets {
    pub material: Handle<StandardMaterial>,
}

impl FromWorld for GobletGraphicsAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(Color::srgb(1.0, 215.0 / 255.0, 0.0));

        Self { material }
    }
}
