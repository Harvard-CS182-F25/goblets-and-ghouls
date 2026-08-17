use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};
use serde::{Deserialize, Serialize};

#[gen_stub_pyclass_enum]
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]

/// Represents a policy for the ghost.
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
/// Stores the configuration of the agent and the ghost, if there is one.
pub struct AgentConfig {
    #[derivative(Default(value = "\"Agent\".to_string()"))]
    pub name: String,

    pub ghost_policy: Option<GhostPolicy>,

    #[derivative(Default(value = "-100"))]
    pub ghost_penalty: i32,

    pub ghost_occlusion: bool,

    pub transition: [f32; 4],
}

#[gen_stub_pymethods]
#[pymethods]
impl AgentConfig {
    /// Returns the name of the agent.
    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[setter]
    fn set_name(&mut self, value: String) {
        self.name = value;
    }

    /// Returns the ghost's policy, if there is one.
    #[getter]
    fn ghost_policy(&self) -> Option<GhostPolicy> {
        self.ghost_policy.clone()
    }

    #[setter]
    fn set_ghost_policy(&mut self, value: Option<GhostPolicy>) {
        self.ghost_policy = value;
    }

    /// Returns the reward applied when the ghost catches the agent.
    #[getter]
    fn ghost_penalty(&self) -> i32 {
        self.ghost_penalty
    }

    #[setter]
    fn set_ghost_penalty(&mut self, value: i32) {
        self.ghost_penalty = value;
    }

    /// When true, a ghost hidden behind a wall (i.e., with no line of sight
    /// from the agent) is not observed for policy look-up purposes. Defaults
    /// to false. Invalid or hypothetical states where the ghost is inside a
    /// wall are also treated as hidden.
    #[getter]
    fn ghost_occlusion(&self) -> bool {
        self.ghost_occlusion
    }

    #[setter]
    fn set_ghost_occlusion(&mut self, value: bool) {
        self.ghost_occlusion = value;
    }

    /// Returns the probabilities that the agent's actual action are
    /// [intended, right, back, left]. Entries must be nonnegative and sum to 1.
    #[getter]
    fn transition(&self) -> [f32; 4] {
        self.transition
    }

    #[setter]
    fn set_transition(&mut self, value: [f32; 4]) {
        self.transition = value;
    }

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
#[pyclass(name = "GobletConfig")]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
/// Stores the configuration of all goblets.
pub struct GobletConfig {
    #[derivative(Default(value = "1"))]
    pub number: usize,

    #[derivative(Default(value = "10"))]
    pub max_reward: u32,
}

#[gen_stub_pymethods]
#[pymethods]
impl GobletConfig {
    /// Returns the initial number of goblets.
    #[getter]
    fn number(&self) -> usize {
        self.number
    }

    #[setter]
    fn set_number(&mut self, value: usize) {
        self.number = value;
    }

    /// Returns the maximum reward of any goblet.
    #[getter]
    fn max_reward(&self) -> u32 {
        self.max_reward
    }

    #[setter]
    fn set_max_reward(&mut self, value: u32) {
        self.max_reward = value;
    }

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

#[gen_stub_pyclass]
#[pyclass(name = "WorldGenerationConfig")]
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(default)]
/// Stores the configuration of the gridworld.
pub struct WorldGenerationConfig {
    #[derivative(Default(value = "100.0"))]
    pub world_width: f32,
    #[derivative(Default(value = "100.0"))]
    pub world_height: f32,
    #[derivative(Default(value = "5"))]
    pub num_obstacles: usize,
    #[derivative(Default(value = "3"))]
    pub obstacle_radius_cells: usize,
    #[derivative(Default(value = "5.0"))]
    pub cell_size: f32,
}

#[gen_stub_pymethods]
#[pymethods]
impl WorldGenerationConfig {
    #[getter]
    fn world_width(&self) -> f32 {
        self.world_width
    }

    #[setter]
    fn set_world_width(&mut self, value: f32) {
        self.world_width = value;
    }

    #[getter]
    fn world_height(&self) -> f32 {
        self.world_height
    }

    #[setter]
    fn set_world_height(&mut self, value: f32) {
        self.world_height = value;
    }

    /// Returns the initial number of obstacles.
    #[getter]
    fn num_obstacles(&self) -> usize {
        self.num_obstacles
    }

    #[setter]
    fn set_num_obstacles(&mut self, value: usize) {
        self.num_obstacles = value;
    }

    /// Returns the maximum radius of an obstacle in number of cells.
    #[getter]
    fn obstacle_radius_cells(&self) -> usize {
        self.obstacle_radius_cells
    }

    #[setter]
    fn set_obstacle_radius_cells(&mut self, value: usize) {
        self.obstacle_radius_cells = value;
    }

    #[getter]
    fn cell_size(&self) -> f32 {
        self.cell_size
    }

    #[setter]
    fn set_cell_size(&mut self, value: f32) {
        self.cell_size = value;
    }

    /// Returns the size of the world as a (width, height) tuple.
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
/// Represents the entire configuration of the game.
pub struct GGConfig {
    #[pyo3(get)]
    pub agent: AgentConfig,
    #[pyo3(get)]
    pub goblets: GobletConfig,
    #[pyo3(get)]
    pub world_generation: WorldGenerationConfig,
    #[pyo3(get, set)]
    #[derivative(Default(value = "1.0"))]
    pub render_delay_secs: f32,
    #[pyo3(get, set)]
    pub generation_seed: Option<u32>,
    /// Optional episode seed used when constructing the initial game state.
    /// This controls the first episode's RNG, but `GameState.reset()` does
    /// not consult it unless a caller explicitly passes that seed back in.
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
// The nested config objects are exposed read-only; mutate via the flattened
// form instead.
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

    #[getter(ghost_penalty)]
    fn get_ghost_penalty(&self) -> i32 {
        self.agent.ghost_penalty
    }
    #[setter(ghost_penalty)]
    fn set_ghost_penalty(&mut self, value: i32) {
        self.agent.ghost_penalty = value;
    }

    #[getter(ghost_occlusion)]
    fn get_ghost_occlusion(&self) -> bool {
        self.agent.ghost_occlusion
    }
    #[setter(ghost_occlusion)]
    fn set_ghost_occlusion(&mut self, value: bool) {
        self.agent.ghost_occlusion = value;
    }

    #[getter(transition)]
    fn get_transition(&self) -> [f32; 4] {
        self.agent.transition
    }
    #[setter(transition)]
    fn set_transition(&mut self, value: [f32; 4]) {
        self.agent.transition = value;
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
