mod components;
mod systems;
mod visual;

use bevy::prelude::*;
use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum};
use serde::{Deserialize, Serialize};

pub use components::*;

use crate::core::{GGConfig, StartupSets};

#[gen_stub_pyclass_enum]
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum GhostPolicy {
    Random,
    Chaser,
}

#[gen_stub_pyclass]
#[pyclass(name = "AgentConfig")]
#[derive(Debug, Clone, Resource, Reflect, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[reflect(Resource)]
#[serde(default)]
pub struct AgentConfig {
    #[pyo3(get, set)]
    #[derivative(Default(value = "\"Agent\".to_string()"))]
    pub name: String,

    #[pyo3(get, set)]
    pub ghost_policy: Option<GhostPolicy>,

    #[pyo3(get, set)]
    pub transition: [f32; 4],
}

#[pymethods]
impl AgentConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("AgentConfig({})", self.__str__()?))
    }

    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to serialize AgentConfig: {}",
                e
            ))
        })
    }
}

pub struct AgentPlugin;
impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<components::PlayerActionMessage>();
        app.add_systems(PreStartup, spawn_agent_assets);
        app.add_systems(Startup, systems::spawn_agents.in_set(StartupSets::Agents));
        app.add_systems(Update, (systems::evaluate_policy, systems::step));
    }
}

fn spawn_agent_assets(mut commands: Commands, config: Res<GGConfig>) {
    if config.headless {
        return;
    }

    commands.init_resource::<visual::AgentGraphicsAssets>();
}
