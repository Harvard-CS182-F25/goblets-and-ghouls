pub mod agent;
pub mod camera;
pub mod coords;
pub mod core;
pub mod debug;
pub mod editor;
pub mod game_state;
pub mod goblet;
pub mod resources;
pub mod scene;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;

use gg_core::{GGConfig, GameState, Policy, ValueGrid};

/// Build a fully-configured Bevy [`App`] for a goblets-and-ghouls session.
/// The caller controls threading/execution — this never calls `.run()`.
///
/// `initial_state` is a fully-constructed board (procedural or loaded from a
/// custom world file); it is inserted as a resource up front rather than
/// regenerated inside Bevy. `value` is the learned value function to render
/// as a heatmap, if one was provided alongside the policy.
pub fn build_app(
    initial_state: GameState,
    config: GGConfig,
    policy: Policy,
    value: Option<ValueGrid>,
) -> App {
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

    app.add_plugins(core::GGPlugin {
        initial_state,
        config,
        policy,
        value,
    });

    app
}

/// Build a fully-configured Bevy [`App`] for the world editor.
///
/// `board` is the initial board state (loaded from a file or created blank).
/// `out_path` is where `S` saves to; if `None`, saves to `load_path` or a
/// generated default name.
pub fn build_editor_app(
    board: editor::EditorBoard,
    out_path: Option<String>,
    load_path: Option<String>,
) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Goblets and Ghouls World Editor".to_string(),
            ..Default::default()
        }),
        close_when_requested: false,
        ..Default::default()
    }));
    app.add_systems(Update, force_focus);
    app.add_plugins(editor::EditorPlugin {
        board,
        out_path,
        load_path,
    });
    app
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
