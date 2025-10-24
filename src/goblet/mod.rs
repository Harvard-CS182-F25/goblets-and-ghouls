mod components;
mod systems;
mod visual;

use bevy::prelude::*;
use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

use crate::core::{GGConfig, StartupSets};

pub use components::*;

#[gen_stub_pyclass]
#[pyclass(name = "GobletConfig")]
#[derive(Debug, Clone, Resource, Reflect, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
#[reflect(Resource)]
pub struct GobletConfig {
    #[pyo3(get, set)]
    #[derivative(Default(value = "1"))]
    pub number: usize,

    #[pyo3(get, set)]
    #[derivative(Default(value = "10"))]
    pub max_reward: u32,
}

#[pymethods]
impl GobletConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("GobletConfig({})", self.__str__()?))
    }

    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to serialize GobletConfig: {}",
                e
            ))
        })
    }
}

pub struct GobletPlugin;
impl Plugin for GobletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_flag_and_capture_point_assets);
        app.add_systems(
            Startup,
            (systems::spawn_goblets).in_set(StartupSets::Goblets),
        );
    }
}

fn init_flag_and_capture_point_assets(mut commands: Commands, config: Res<GGConfig>) {
    if !config.headless {
        commands.init_resource::<visual::GobletGraphicsAssets>();
    }
}
