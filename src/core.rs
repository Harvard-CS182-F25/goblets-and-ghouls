use bevy::prelude::*;
use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

use crate::agent;
use crate::agent::Action;
use crate::camera;
use crate::game_state;
use crate::goblet;
use crate::scene;

#[derive(SystemSet, Debug, Clone, Hash, PartialEq, Eq)]
pub enum StartupSets {
    Agents,
    Goblets,
    Walls,
}

#[gen_stub_pyclass]
#[pyclass(name = "GGConfig")]
#[derive(Debug, Clone, Resource, Reflect, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
#[reflect(Resource)]
pub struct GGConfig {
    #[pyo3(get, set)]
    pub agent: agent::AgentConfig,
    #[pyo3(get, set)]
    pub camera: camera::CameraConfig,
    #[pyo3(get, set)]
    pub goblets: goblet::GobletConfig,
    #[pyo3(get, set)]
    pub world_generation: scene::WorldGenerationConfig,
    #[pyo3(get, set)]
    #[derivative(Default(value = "1.0"))]
    pub render_delay_secs: f32,
    #[pyo3(get, set)]
    pub generation_seed: Option<u32>,
    #[pyo3(get, set)]
    pub episode_seed: Option<u32>,
    #[pyo3(get, set)]
    pub debug: bool,
    #[pyo3(get, set)]
    pub headless: bool,
}

#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct Policy(pub Vec<Action>);

#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct PolicyTimer(pub Timer);

#[pymethods]
impl GGConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("GGConfig({})", self.__str__()?))
    }

    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to serialize GGConfig: {}",
                e
            ))
        })
    }
}

pub struct GGPlugin {
    pub config: GGConfig,
    pub policy: Vec<Action>,
}

impl Plugin for GGPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
        app.insert_resource(Policy(self.policy.clone()));
        app.insert_resource(PolicyTimer(Timer::from_seconds(
            self.config.render_delay_secs,
            TimerMode::Repeating,
        )));

        app.add_plugins((
            agent::AgentPlugin,
            camera::CameraPlugin,
            goblet::GobletPlugin,
            scene::ScenePlugin,
            game_state::GameStatePlugin,
        ));

        app.configure_sets(
            Startup,
            (
                StartupSets::Walls,
                StartupSets::Goblets,
                StartupSets::Agents,
            )
                .chain(),
        );

        app.insert_resource(ClearColor(Color::srgb_u8(0, 136, 255)));
    }
}
