use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};
use serde::{Deserialize, Serialize};

#[gen_stub_pyclass_enum]
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GhostPolicy {
    Random,
    Chaser,
    /// The ghost's movement is supplied externally each tick (e.g. by a
    /// human at the keyboard in the live game) rather than decided
    /// automatically — see `GameState::step_teleop`. Not usable in headless
    /// mode (there's no human input to drive it during automated training).
    Teleop,
}

#[gen_stub_pyclass]
#[pyclass(name = "AgentConfig")]
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(default)]
pub struct AgentConfig {
    #[pyo3(get, set)]
    #[derivative(Default(value = "\"Agent\".to_string()"))]
    pub name: String,

    #[pyo3(get, set)]
    pub ghost_policy: Option<GhostPolicy>,

    #[pyo3(get, set)]
    pub transition: [f32; 4],

    /// When true, a ghost hidden behind a wall (no line of sight from the
    /// agent) is treated as if it were at the agent's own position for
    /// policy-lookup purposes, the same convention used when no ghost exists.
    #[pyo3(get, set)]
    pub ghost_occlusion: bool,
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

#[gen_stub_pyclass]
#[pyclass(name = "CameraConfig")]
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(default)]
pub struct CameraConfig {
    #[pyo3(get, set)]
    #[derivative(Default(value = "-0.15"))]
    pub scale: f32,
}

#[pymethods]
impl CameraConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("CameraConfig({})", self.__str__()?))
    }

    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to serialize CameraConfig: {}",
                e
            ))
        })
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "GobletConfig")]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
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

/// Procedural world-generation parameters. Ignored (except `cell_size`, which
/// still controls rendering scale) when `GGConfig.world_file` is set.
#[gen_stub_pyclass]
#[pyclass(name = "WorldGenerationConfig")]
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
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

#[gen_stub_pyclass]
#[pyclass(name = "GGConfig")]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct GGConfig {
    #[pyo3(get, set)]
    pub agent: AgentConfig,
    #[pyo3(get, set)]
    pub camera: CameraConfig,
    #[pyo3(get, set)]
    pub goblets: GobletConfig,
    #[pyo3(get, set)]
    pub world_generation: WorldGenerationConfig,
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
    /// Path to a YAML world file to load instead of procedurally generating
    /// a board. When set, `world_generation`'s `world_width`/`world_height`/
    /// `num_obstacles`/`obstacle_radius_cells` are ignored (the board's
    /// dimensions/walls/goblets/agent/ghost positions come from the file);
    /// `world_generation.cell_size` still applies for rendering scale.
    #[pyo3(get, set)]
    pub world_file: Option<String>,
}

// PyO3 can't expose a nested `#[pyclass]` field (e.g. `agent: AgentConfig`)
// as a live reference into the parent — `config.agent` clones it into a new
// Python object every access, so `config.agent.ghost_occlusion = False`
// silently mutates a throwaway clone instead of `config` itself. Rather than
// switching to `Py<T>` fields (which would force every pure-Rust read of
// `config.agent.*` elsewhere in this crate to acquire the Python GIL,
// breaking `gg_core`'s use as a standalone engine), the nested leaf fields
// people actually script against are flattened directly onto `GGConfig` —
// e.g. `config.ghost_occlusion = False` is a real one-line mutation.
// `config.agent.ghost_occlusion = False` remains silently ineffective; use
// the flattened form (or reassign the whole `agent` object) instead.
#[gen_stub_pymethods]
#[pymethods]
impl GGConfig {
    #[getter(name)]
    fn get_name(&self) -> String {
        self.agent.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: String) {
        self.agent.name = value;
    }

    #[getter(ghost_policy)]
    fn get_ghost_policy(&self) -> Option<GhostPolicy> {
        self.agent.ghost_policy.clone()
    }
    #[setter(ghost_policy)]
    fn set_ghost_policy(&mut self, value: Option<GhostPolicy>) {
        self.agent.ghost_policy = value;
    }

    #[getter(transition)]
    fn get_transition(&self) -> [f32; 4] {
        self.agent.transition
    }
    #[setter(transition)]
    fn set_transition(&mut self, value: [f32; 4]) {
        self.agent.transition = value;
    }

    #[getter(ghost_occlusion)]
    fn get_ghost_occlusion(&self) -> bool {
        self.agent.ghost_occlusion
    }
    #[setter(ghost_occlusion)]
    fn set_ghost_occlusion(&mut self, value: bool) {
        self.agent.ghost_occlusion = value;
    }

    #[getter(scale)]
    fn get_scale(&self) -> f32 {
        self.camera.scale
    }
    #[setter(scale)]
    fn set_scale(&mut self, value: f32) {
        self.camera.scale = value;
    }

    #[getter(number)]
    fn get_number(&self) -> usize {
        self.goblets.number
    }
    #[setter(number)]
    fn set_number(&mut self, value: usize) {
        self.goblets.number = value;
    }

    #[getter(max_reward)]
    fn get_max_reward(&self) -> u32 {
        self.goblets.max_reward
    }
    #[setter(max_reward)]
    fn set_max_reward(&mut self, value: u32) {
        self.goblets.max_reward = value;
    }

    #[getter(world_width)]
    fn get_world_width(&self) -> f32 {
        self.world_generation.world_width
    }
    #[setter(world_width)]
    fn set_world_width(&mut self, value: f32) {
        self.world_generation.world_width = value;
    }

    #[getter(world_height)]
    fn get_world_height(&self) -> f32 {
        self.world_generation.world_height
    }
    #[setter(world_height)]
    fn set_world_height(&mut self, value: f32) {
        self.world_generation.world_height = value;
    }

    #[getter(num_obstacles)]
    fn get_num_obstacles(&self) -> usize {
        self.world_generation.num_obstacles
    }
    #[setter(num_obstacles)]
    fn set_num_obstacles(&mut self, value: usize) {
        self.world_generation.num_obstacles = value;
    }

    #[getter(obstacle_radius_cells)]
    fn get_obstacle_radius_cells(&self) -> usize {
        self.world_generation.obstacle_radius_cells
    }
    #[setter(obstacle_radius_cells)]
    fn set_obstacle_radius_cells(&mut self, value: usize) {
        self.world_generation.obstacle_radius_cells = value;
    }

    #[getter(cell_size)]
    fn get_cell_size(&self) -> f32 {
        self.world_generation.cell_size
    }
    #[setter(cell_size)]
    fn set_cell_size(&mut self, value: f32) {
        self.world_generation.cell_size = value;
    }

    /// The size of the board as (width, height) — same as
    /// `config.world_generation.size`, flattened for convenience. Only
    /// reflects the *procedurally-generated* size; when `world_file` is
    /// set, read the actual board size from `GameState.board.width/height`
    /// instead.
    #[getter(size)]
    fn get_size(&self) -> (usize, usize) {
        self.world_generation.size()
    }

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
