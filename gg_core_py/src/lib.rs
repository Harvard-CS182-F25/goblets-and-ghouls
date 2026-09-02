#[cfg(feature = "bevy")]
mod bevy_runner;

#[cfg(feature = "bevy")]
use pyo3::exceptions::PyTypeError;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};
use rand::SeedableRng;
use wyrand::WyRand;

use gg_core::{Action, Board, GGConfig, GameState, GhostPolicy, WorldFile};

#[gen_stub_pyfunction]
#[pyfunction(name = "parse_config")]
fn parse_config(config_path: &str) -> PyResult<GGConfig> {
    let config_str = std::fs::read_to_string(config_path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read config file: {}", e)))?;

    let config: GGConfig = serde_yaml::from_str(&config_str)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse config file: {}", e)))?;

    Ok(config)
}

#[gen_stub_pyfunction]
#[pyfunction(name = "run")]
#[pyo3(signature=(config, policy=None, value=None))]
#[cfg_attr(not(feature = "bevy"), allow(unused_variables))]
fn run(
    py: Python<'_>,
    mut config: GGConfig,
    policy: Option<Py<PyAny>>,
    value: Option<Py<PyAny>>,
) -> PyResult<Option<(GameState, u32, u64)>> {
    if config.headless && config.agent.ghost_policy == Some(GhostPolicy::Teleop) {
        return Err(PyValueError::new_err(
            "GhostPolicy::Teleop requires a human at the keyboard and cannot be used in \
             headless mode (e.g. during training)",
        ));
    }

    let generation_seed = if let Some(seed) = config.world_generation.seed {
        seed
    } else {
        let seed = rand::random::<u32>();
        config.world_generation.seed = Some(seed);
        seed
    };

    let board = if let Some(path) = config.world_file.clone() {
        WorldFile::from_file(&path)
            .and_then(|w| w.into_board())
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to load world file: {e}")))?
    } else {
        let mut rng = WyRand::from_seed(u64::from(generation_seed).to_ne_bytes());
        Board::new(&mut rng, &config)
    };

    let mut initial_state = GameState::from(board).with_config(&config);

    if let Some(episode_seed) = config.episode_seed {
        initial_state = initial_state.with_seed(episode_seed.into());
    }

    let episode_seed = initial_state.rng_seed;

    if !config.headless {
        #[cfg(feature = "bevy")]
        {
            let policy_any = policy.ok_or_else(|| {
                PyTypeError::new_err("Policy must be provided in non-headless mode")
            })?;
            let policy = numpy_to_policy(py, &policy_any)?;
            let value = value
                .map(|value_any| numpy_to_value_grid(py, &value_any))
                .transpose()?;

            gg_bevy::build_app(initial_state, config, policy, value).run();
            Ok(None)
        }

        #[cfg(not(feature = "bevy"))]
        {
            let _ = policy;
            let _ = value;
            Err(PyRuntimeError::new_err(
                "This build of gg_core was compiled without the `bevy` feature; non-headless mode is unavailable",
            ))
        }
    } else {
        Ok(Some((initial_state, generation_seed, episode_seed)))
    }
}

/// Converts an incoming policy array into a [`gg_core::Policy`].
///
/// Two shapes are accepted:
/// - 4D `(width, height, width, height)`, indexed `[ax, ay, gx, gy]` — a
///   genuine ghost-position-dependent policy, matching the axis order a
///   Q-table already uses (`q[ax, ay, gx, gy, action]`).
/// - 2D `(width, height)`, indexed `[ax, ay]` — a ghost-free policy
///   (e.g. from Value Iteration),
///   broadcast across every ghost slice via [`gg_core::Policy::from_agent_grid`]
///   so downstream code can treat both shapes uniformly.
#[cfg(feature = "bevy")]
fn numpy_to_policy(py: Python<'_>, policy_any: &Py<PyAny>) -> PyResult<gg_core::Policy> {
    use numpy::{PyArray2, PyArray4, PyArrayMethods};

    if let Ok(arr4) = policy_any.cast_bound::<PyArray4<Py<PyAny>>>(py) {
        let array = unsafe { arr4.as_array() };
        let shape = array.shape();
        let (width, height, width2, height2) = (shape[0], shape[1], shape[2], shape[3]);
        if width != width2 || height != height2 {
            return Err(PyTypeError::new_err(format!(
                "4D policy array must have shape (width, height, width, height); got {shape:?}"
            )));
        }

        let mut policy = gg_core::Policy::new(width, height, Action::Up);
        for ax in 0..width {
            for ay in 0..height {
                for gx in 0..width {
                    for gy in 0..height {
                        let item = array.get([ax, ay, gx, gy]).unwrap();
                        let action: Action = item.extract(py)?;
                        policy.set((ax, ay), (gx, gy), action);
                    }
                }
            }
        }
        return Ok(policy);
    }

    let arr2 = policy_any.cast_bound::<PyArray2<Py<PyAny>>>(py).map_err(|_| {
        PyTypeError::new_err(
            "Policy must be a 2D (width, height) or 4D (width, height, width, height) numpy.ndarray",
        )
    })?;
    let array = unsafe { arr2.as_array() };
    let width = array.shape()[0];
    let height = array.shape()[1];
    let mut agent_actions: Vec<Action> = Vec::with_capacity(width * height);
    for ay in 0..height {
        for ax in 0..width {
            let item = array.get([ax, ay]).unwrap();
            let action: Action = item.extract(py)?;
            agent_actions.push(action);
        }
    }

    Ok(gg_core::Policy::from_agent_grid(
        agent_actions,
        width,
        height,
    ))
}

/// Converts an incoming value-function array into a [`gg_core::ValueGrid`].
/// Same dual-shape handling as [`numpy_to_policy`], but for plain `float32`
/// arrays — no per-element Python object extraction needed.
#[cfg(feature = "bevy")]
fn numpy_to_value_grid(py: Python<'_>, value_any: &Py<PyAny>) -> PyResult<gg_core::ValueGrid> {
    use numpy::{PyArray2, PyArray4, PyArrayMethods};

    if let Ok(arr4) = value_any.cast_bound::<PyArray4<f32>>(py) {
        let array = unsafe { arr4.as_array() };
        let shape = array.shape();
        let (width, height, width2, height2) = (shape[0], shape[1], shape[2], shape[3]);
        if width != width2 || height != height2 {
            return Err(PyTypeError::new_err(format!(
                "4D value array must have shape (width, height, width, height); got {shape:?}"
            )));
        }

        let mut values = gg_core::ValueGrid::new(width, height, 0.0);
        for ax in 0..width {
            for ay in 0..height {
                for gx in 0..width {
                    for gy in 0..height {
                        values.set((ax, ay), (gx, gy), array[[ax, ay, gx, gy]]);
                    }
                }
            }
        }
        return Ok(values);
    }

    let arr2 = value_any.cast_bound::<PyArray2<f32>>(py).map_err(|_| {
        PyTypeError::new_err(
            "Value function must be a 2D (width, height) or 4D (width, height, width, height) numpy.ndarray of float32",
        )
    })?;
    let array = unsafe { arr2.as_array() };
    let width = array.shape()[0];
    let height = array.shape()[1];
    let mut agent_values: Vec<f32> = Vec::with_capacity(width * height);
    for ay in 0..height {
        for ax in 0..width {
            agent_values.push(array[[ax, ay]]);
        }
    }

    Ok(gg_core::ValueGrid::from_agent_grid(
        agent_values,
        width,
        height,
    ))
}

#[pymodule]
fn _core(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(parse_config, m)?)?;

    m.add_class::<GGConfig>()?;
    m.add_class::<Action>()?;
    m.add_class::<gg_core::AgentConfig>()?;
    m.add_class::<GameState>()?;
    m.add_class::<gg_core::EntityType>()?;
    m.add_class::<gg_core::WorldGenerationConfig>()?;

    #[cfg(feature = "bevy")]
    {
        m.add_function(wrap_pyfunction!(bevy_runner::run_editor, m)?)?;
    }

    Ok(())
}

define_stub_info_gatherer!(stub_info);
