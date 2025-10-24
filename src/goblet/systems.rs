use bevy::prelude::*;

use crate::core::GGConfig;
use crate::game_state::{GameState, Goblet};
use crate::goblet::GobletBundle;

use super::visual::GobletGraphicsAssets;

pub fn cell_to_world(
    position: (usize, usize),
    cell_size: f32,
    world_width: f32,
    world_height: f32,
) -> Vec3 {
    Vec3::new(
        position.0 as f32 * cell_size + cell_size / 2.0 - world_width / 2.0,
        0.0,
        position.1 as f32 * cell_size + cell_size / 2.0 - world_height / 2.0,
    )
}

pub fn spawn_goblets(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    goblet_graphics: Option<Res<GobletGraphicsAssets>>,
    state: Res<GameState>,
    config: Res<GGConfig>,
) {
    for (i, &Goblet { position, reward }) in state.board.goblets.iter().enumerate() {
        let goblet_name = format!("Goblet {}", i + 1);
        let world_position = cell_to_world(
            position,
            config.world_generation.cell_size,
            config.world_generation.world_width,
            config.world_generation.world_height,
        );
        info!("Spawning Goblet at world position: {world_position} {position:?}");

        let mut entity = commands.spawn(GobletBundle::new(&goblet_name, world_position, reward));

        if let Some(goblet_graphics) = &goblet_graphics
            && let Some(meshes_ref) = &mut meshes
        {
            let mesh = meshes_ref.add(Cylinder::new(config.world_generation.cell_size / 2.0, 3.0));
            let material = if reward > 0 {
                goblet_graphics.material.clone()
            } else {
                goblet_graphics.false_material.clone()
            };

            entity.insert((Mesh3d(mesh.clone()), MeshMaterial3d(material)));
        }
    }
}
