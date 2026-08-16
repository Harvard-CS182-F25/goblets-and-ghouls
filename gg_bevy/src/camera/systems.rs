use bevy::prelude::*;
use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};

use crate::camera::{CameraZoomLimits, PanState, drag_delta, pan_and_clamp, shift_held};
use crate::coords::world_dimensions;
use crate::resources::{ConfigResource, GameStateResource};

use super::PAN_PADDING_CELLS;

pub fn setup_camera(
    mut commands: Commands,
    config: Res<ConfigResource>,
    state: Res<GameStateResource>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if config.0.headless {
        return;
    }

    let scale = config.0.camera.scale;
    if let Ok(window) = windows.single() {
        let cell_size = config.0.world_generation.cell_size;
        let (board_width, board_height) =
            world_dimensions(state.0.board.width, state.0.board.height, cell_size);
        commands.insert_resource(CameraZoomLimits::new(
            Vec2::new(board_width, board_height),
            cell_size,
            Vec2::new(window.width(), window.height()),
            scale,
        ));
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        Projection::from(OrthographicProjection {
            scale,
            ..OrthographicProjection::default_3d()
        }),
    ));
}

pub fn zoom_in(limits: Res<CameraZoomLimits>, mut query: Query<&mut Projection, With<Camera3d>>) {
    for mut proj in query.iter_mut() {
        if let Projection::Orthographic(ortho) = &mut *proj {
            let scale = ortho.scale.abs();
            if scale > limits.min_scale {
                ortho.scale = ortho.scale.signum() * (scale * 0.97).max(limits.min_scale);
            }
        }
    }
}

pub fn zoom_out(limits: Res<CameraZoomLimits>, mut query: Query<&mut Projection, With<Camera3d>>) {
    for mut proj in query.iter_mut() {
        if let Projection::Orthographic(ortho) = &mut *proj {
            let scale = ortho.scale.abs();
            if scale < limits.max_scale {
                ortho.scale = ortho.scale.signum() * (scale / 0.97).min(limits.max_scale);
            }
        }
    }
}

/// Swaps the OS cursor to an open/closed hand while Shift (the pan modifier)
/// is held. The editor's blocking exit dialog disables this affordance because
/// panning is unavailable there.
pub fn update_pan_cursor(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    window_q: Query<Entity, With<PrimaryWindow>>,
    editor_state: Option<Res<crate::editor::EditorState>>,
    mut current: Local<Option<SystemCursorIcon>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let pan_available = editor_state.is_none_or(|state| !state.exit_dialog_open);
    let desired = (pan_available && shift_held(&keys)).then(|| {
        if mouse_btn.pressed(MouseButton::Left) {
            SystemCursorIcon::Grabbing
        } else {
            SystemCursorIcon::Grab
        }
    });
    if desired == *current {
        return;
    }
    *current = desired;
    match desired {
        Some(icon) => {
            commands.entity(window).insert(CursorIcon::System(icon));
        }
        None => {
            commands.entity(window).remove::<CursorIcon>();
        }
    }
}

/// Shift+Left-click drag to pan (arrow keys/WASD are reserved for
/// teleop-controlled ghost movement instead — see `agent::systems::evaluate_policy_teleop`).
pub fn pan_camera(
    mut query: Query<(&mut Transform, &Projection), With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    game_state: Res<GameStateResource>,
    config: Res<ConfigResource>,
    mut pan_state: ResMut<PanState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let dragging = shift_held(&keyboard_input) && mouse_button.pressed(MouseButton::Left);
    let delta = drag_delta(&mut pan_state, dragging, window.cursor_position());

    let cell_size = config.0.world_generation.cell_size;
    let (board_w, board_h) =
        world_dimensions(game_state.0.board.width, game_state.0.board.height, cell_size);
    let board_size = Vec2::new(board_w, board_h);
    let window_size = Vec2::new(window.width(), window.height());
    let padding = cell_size * PAN_PADDING_CELLS;

    for (mut transform, projection) in query.iter_mut() {
        let Projection::Orthographic(ortho) = projection else {
            continue;
        };
        // Screen-space delta -> world-space: `scale` converts pixels to
        // world units, so the point under the cursor stays under it while
        // dragging. Z takes the screen Y delta (gg's ground plane is XZ,
        // viewed top-down). X is *not* negated here (unlike a typical 2D
        // camera) — this camera's NEG_Z up-vector mirrors the X axis on
        // screen relative to world X, so a positive world-X translation is
        // what makes the world track the cursor correctly while dragging.
        let direction = Vec3::new(delta.x * ortho.scale, 0.0, delta.y * ortho.scale);
        pan_and_clamp(
            &mut transform.translation,
            direction,
            ortho.scale,
            window_size,
            board_size,
            padding,
            Vec2::ZERO,
        );
    }
}
