use bevy::prelude::*;

/// Meshes shared by all game entities of the same kind.
#[derive(Resource)]
pub struct RenderMeshAssets {
    pub wall: Handle<Mesh>,
    pub agent: Handle<Mesh>,
    pub ghost: Handle<Mesh>,
    pub goblet: Handle<Mesh>,
}

#[derive(Resource)]
pub struct WallGraphicsAssets {
    pub material: Handle<StandardMaterial>,
}

impl FromWorld for WallGraphicsAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material: Handle<StandardMaterial> = materials.add(Color::srgb(0.0, 0.0, 0.0));

        Self { material }
    }
}
