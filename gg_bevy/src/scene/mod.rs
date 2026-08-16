mod components;
mod systems;
mod visual;

use bevy::prelude::*;

pub use components::*;
pub use visual::*;

use crate::core::StartupSets;
use crate::resources::ConfigResource;

pub const AGENT_HEIGHT: f32 = 4.0;
pub const GHOST_HEIGHT: f32 = 4.0;
pub const GOBLET_HEIGHT: f32 = 3.0;
pub const WALL_HEIGHT: f32 = 5.0;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1_500.0,
            ..Default::default()
        });

        app.add_systems(PreStartup, init_wall_assets);
        app.add_systems(
            Startup,
            (
                (systems::setup_scene, systems::spawn_walls).in_set(StartupSets::Walls),
                systems::setup_key_instructions,
            ),
        );
        app.add_systems(
            Update,
            systems::update_hud_text.run_if(|config: Res<ConfigResource>| !config.0.headless),
        );
    }
}

fn init_wall_assets(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    config: Res<ConfigResource>,
) {
    if !config.0.headless {
        commands.init_resource::<WallGraphicsAssets>();
        let cell = config.0.world_generation.cell_size;
        if let Some(meshes) = &mut meshes {
            commands.insert_resource(RenderMeshAssets {
                wall: meshes.add(Cuboid::new(cell, WALL_HEIGHT, cell)),
                agent: meshes.add(Cuboid::new(cell, AGENT_HEIGHT, cell)),
                ghost: meshes.add(Cuboid::new(cell, GHOST_HEIGHT, cell)),
                goblet: meshes.add(Cylinder::new(cell / 2.0, GOBLET_HEIGHT)),
            });
        }
    }
}
