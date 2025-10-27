mod agent;
mod camera;
mod core;
mod debug;
mod game_state;
mod goblet;
mod scene;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use numpy::{PyArray2, PyArrayMethods};
use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};
use rand::SeedableRng;

use crate::{
    core::GGConfig,
    game_state::{Board, GameState},
};

#[gen_stub_pyfunction]
#[pyfunction(name = "parse_config")]
fn parse_config(config_path: &str) -> PyResult<GGConfig> {
    let config_str = std::fs::read_to_string(config_path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read config file: {}", e)))?;

    let config: GGConfig = serde_yaml::from_str(&config_str)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse config file: {}", e)))?;

    Ok(config)
}

fn generate_app(mut config: GGConfig, policy: Vec<agent::Action>) -> App {
    let mut app = App::new();

    if !config.headless {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ghouls and Goblets".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }));

        app.add_systems(Update, force_focus);
    }

    if config.debug {
        app.add_plugins(debug::DebugPlugin);
    }

    let seed = if let Some(seed) = config.generation_seed {
        seed
    } else {
        let seed = rand::random::<u16>().into();
        config.generation_seed = Some(seed);
        seed
    };

    app.add_plugins((
        EntropyPlugin::<WyRand>::with_seed(u64::from(seed).to_ne_bytes()),
        core::GGPlugin {
            config: config.clone(),
            policy,
        },
    ));

    app
}

#[gen_stub_pyfunction]
#[pyfunction(name = "run")]
#[pyo3(signature=(config, policy=None))]
fn run(
    py: Python<'_>,
    mut config: GGConfig,
    policy: Option<Py<PyAny>>,
) -> PyResult<Option<(GameState, u32, u64)>> {
    if !config.headless {
        let policy_any = policy
            .ok_or_else(|| PyTypeError::new_err("Policy must be provided in non-headless mode"))?;

        let policy = if let Ok(arr_obj) = policy_any.cast_bound::<PyArray2<Py<PyAny>>>(py) {
            let array = unsafe { arr_obj.as_array() };
            let n_rows = array.shape()[0];
            let n_cols = array.shape()[1];
            let mut policy_vec: Vec<agent::Action> = Vec::with_capacity(n_rows * n_cols);
            for row in 0..n_rows {
                for col in 0..n_cols {
                    let item = array.get([col, row]).unwrap();
                    let action: agent::Action = item.extract(py)?;
                    policy_vec.push(action);
                }
            }

            Ok(policy_vec)
        } else {
            Err(PyTypeError::new_err(
                "Policy must be a numpy.ndarray in non-headless mode",
            ))
        }?;

        let mut app = generate_app(config, policy);
        app.run();
        Ok(None)
    } else {
        let generation_seed = if let Some(seed) = config.generation_seed {
            seed
        } else {
            let seed = rand::random::<u16>().into();
            config.generation_seed = Some(seed);
            seed
        };
        let mut rng = WyRand::from_seed(u64::from(generation_seed).to_ne_bytes());
        let mut initial_state = GameState::from(Board::new(&mut rng, &config)).with_config(&config);

        if let Some(episode_seed) = config.episode_seed {
            initial_state = initial_state.with_seed(episode_seed.into());
        }

        let episode_seed = initial_state.rng_seed;

        Ok(Some((initial_state, generation_seed, episode_seed)))
    }
}

fn force_focus(
    mut done: Local<bool>,
    winit: Option<NonSend<WinitWindows>>,
    primary: Query<Entity, With<PrimaryWindow>>,
) {
    if *done {
        return;
    }
    let Some(winit) = winit else { return }; // not present on wasm
    let Ok(win_entity) = primary.single() else {
        return;
    };
    if let Some(w) = winit.get_window(win_entity) {
        w.focus_window();
    }
    *done = true;
}

#[pymodule]
fn _core(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(parse_config, m)?)?;

    m.add_class::<core::GGConfig>()?;
    m.add_class::<agent::Action>()?;
    m.add_class::<agent::AgentConfig>()?;
    m.add_class::<camera::CameraConfig>()?;
    m.add_class::<game_state::GameState>()?;
    m.add_class::<game_state::EntityType>()?;
    m.add_class::<scene::WorldGenerationConfig>()?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
