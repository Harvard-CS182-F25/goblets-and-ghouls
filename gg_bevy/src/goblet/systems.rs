use bevy::prelude::*;
use gg_core::Goblet as GobletData;

use crate::coords::{cell_to_world, world_dimensions};
use crate::goblet::GobletBundle;
use crate::resources::{ConfigResource, GameStateResource};
use crate::scene::GOBLET_HEIGHT;

use super::visual::GobletGraphicsAssets;

pub fn spawn_goblets(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    goblet_graphics: Option<Res<GobletGraphicsAssets>>,
    state: Res<GameStateResource>,
    config: Res<ConfigResource>,
) {
    let cell = config.0.world_generation.cell_size;
    let (world_width, world_height) =
        world_dimensions(state.0.board.width, state.0.board.height, cell);

    for (i, &GobletData { position, reward }) in state.0.board.goblets.iter().enumerate() {
        let goblet_name = format!("Goblet {}", i + 1);
        let world_position =
            cell_to_world(position, cell, world_width, world_height, GOBLET_HEIGHT / 2.0);

        let mut entity = commands.spawn(GobletBundle::new(&goblet_name, world_position, reward));

        if let Some(goblet_graphics) = &goblet_graphics
            && let Some(meshes_ref) = &mut meshes
        {
            let mesh = meshes_ref.add(Cylinder::new(cell / 2.0, GOBLET_HEIGHT));
            let material = if reward > 0 {
                goblet_graphics.material.clone()
            } else {
                goblet_graphics.false_material.clone()
            };

            entity.insert((Mesh3d(mesh.clone()), MeshMaterial3d(material)));
        }
    }
}
