use bevy::prelude::*;

#[derive(Resource)]
pub struct GobletGraphicsAssets {
    pub material: Handle<StandardMaterial>,
    pub false_material: Handle<StandardMaterial>,
}

impl FromWorld for GobletGraphicsAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(Color::srgb_u8(255, 215, 0));
        let false_material = materials.add(Color::srgb_u8(255, 69, 0));

        Self {
            material,
            false_material,
        }
    }
}
