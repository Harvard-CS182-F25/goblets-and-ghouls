use bevy::prelude::*;

#[derive(Resource)]
pub struct AgentGraphicsAssets {
    pub material: Handle<StandardMaterial>,
    pub ghost_material: Handle<StandardMaterial>,
}

impl FromWorld for AgentGraphicsAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material: Handle<StandardMaterial> = materials.add(Color::srgb(1.0, 0.0, 0.0));
        let ghost_material: Handle<StandardMaterial> = materials.add(Color::srgb(1.0, 1.0, 1.0));

        Self {
            material,
            ghost_material,
        }
    }
}
