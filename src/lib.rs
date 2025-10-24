mod agent;
mod camera;
mod core;
mod debug;
mod game_state;
mod goblet;
mod scene;

use bevy::prelude::*;
use bevy::window::WindowCreated;
use bevy::winit::WinitWindows;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};

use crate::core::GGConfig;

#[gen_stub_pyfunction]
#[pyfunction(name = "parse_config")]
fn parse_config(config_path: &str) -> PyResult<GGConfig> {
    let config_str = std::fs::read_to_string(config_path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read config file: {}", e)))?;

    let config: GGConfig = serde_yaml::from_str(&config_str)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse config file: {}", e)))?;

    Ok(config)
}

fn generate_app(config: &mut GGConfig) -> App {
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

    let seed = if let Some(seed) = config.seed {
        seed
    } else {
        let seed = rand::random::<u16>().into();
        config.seed = Some(seed);
        seed
    };

    app.add_plugins((
        EntropyPlugin::<WyRand>::with_seed(u64::from(seed).to_ne_bytes()),
        core::GGPlugin {
            config: config.clone(),
        },
    ));

    app
}

#[gen_stub_pyfunction]
#[pyfunction(name = "run")]
fn run(mut config: GGConfig) -> PyResult<()> {
    let mut app = generate_app(&mut config);
    app.run();
    Ok(())
}

fn force_focus(
    winit_windows: Option<NonSend<WinitWindows>>,
    mut created: MessageReader<WindowCreated>,
) {
    let Some(winit_windows) = winit_windows else {
        return;
    };

    for ev in created.read() {
        if let Some(win) = winit_windows.get_window(ev.window) {
            win.focus_window();
        }
    }
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
