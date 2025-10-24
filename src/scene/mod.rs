mod components;
mod systems;
mod visual;

use bevy::prelude::*;
use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Serialize};

pub use components::*;
pub use visual::*;

use crate::core::{GGConfig, StartupSets};

pub const WALL_HEIGHT: f32 = 5.0;

#[gen_stub_pyclass]
#[pyclass(name = "WorldGenerationConfig")]
#[derive(Debug, Clone, Resource, Reflect, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[reflect(Resource)]
#[serde(default)]
pub struct WorldGenerationConfig {
    #[pyo3(get, set)]
    #[derivative(Default(value = "100.0"))]
    pub world_width: f32,
    #[pyo3(get, set)]
    #[derivative(Default(value = "100.0"))]
    pub world_height: f32,
    #[pyo3(get, set)]
    #[derivative(Default(value = "5"))]
    pub num_obstacles: usize,
    #[pyo3(get, set)]
    #[derivative(Default(value = "3"))]
    pub obstacle_radius_cells: usize,
    #[pyo3(get, set)]
    #[derivative(Default(value = "5.0"))]
    pub cell_size: f32,
}

#[gen_stub_pymethods]
#[pymethods]
impl WorldGenerationConfig {
    /// Returns the size of the maze as (width, height)
    #[getter]
    fn size(&self) -> (usize, usize) {
        (
            (self.world_width / self.cell_size).round() as usize,
            (self.world_height / self.cell_size).round() as usize,
        )
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("MazeGenerationConfig({})", self.__str__()?))
    }

    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to serialize MazeGenerationConfig: {}",
                e
            ))
        })
    }
}

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
                (systems::spawn_seed_text, systems::setup_key_instructions),
            ),
        );
    }
}

fn init_wall_assets(mut commands: Commands, config: Res<GGConfig>) {
    if !config.headless {
        commands.init_resource::<WallGraphicsAssets>();
    }
}
