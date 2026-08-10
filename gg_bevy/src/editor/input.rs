use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::window::WindowCloseRequested;

use crate::camera::{PAN_PADDING_CELLS, PanState, drag_delta, pan_and_clamp, shift_held};
use crate::coords::raycast_to_grid_cell;
use crate::scene::GroundPlane;

use super::ui::{ExitDialogButton, FilenameInputBox};
use super::{CELL_SIZE, EditorBoard, EditorState, EditorTile, save_board};

pub fn handle_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    board: Res<EditorBoard>,
    mut exit: MessageWriter<AppExit>,
) {
    // While the exit-confirm dialog is up, only Escape (cancel) does anything.
    if state.exit_dialog_open {
        if keys.just_pressed(KeyCode::Escape) {
            state.exit_dialog_open = false;
        }
        return;
    }

    // While the filename box has focus, suppress all tool shortcuts.
    if state.filename_editing {
        return;
    }

    if keys.just_pressed(KeyCode::KeyE) {
        state.current_tool = EditorTile::Empty;
    }
    if keys.just_pressed(KeyCode::KeyW) {
        state.current_tool = EditorTile::Wall;
    }
    if keys.just_pressed(KeyCode::Digit1) {
        state.current_tool = EditorTile::Agent;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.current_tool = EditorTile::Ghost;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.current_tool = EditorTile::Goblet(state.current_goblet_reward);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        state.current_goblet_reward -= 1;
        if let EditorTile::Goblet(_) = state.current_tool {
            state.current_tool = EditorTile::Goblet(state.current_goblet_reward);
        }
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        state.current_goblet_reward += 1;
        if let EditorTile::Goblet(_) = state.current_tool {
            state.current_tool = EditorTile::Goblet(state.current_goblet_reward);
        }
    }
    if keys.just_pressed(KeyCode::KeyG) {
        state.show_grid = !state.show_grid;
    }
    if keys.just_pressed(KeyCode::Escape) {
        if state.dirty {
            state.exit_dialog_open = true;
        } else {
            exit.write(AppExit::Success);
        }
    }
    if keys.just_pressed(KeyCode::KeyS) {
        save_board(&board, &mut state);
    }
}

/// Handles the operating system's window-close request the same way as
/// Escape: dirty boards require confirmation, while clean boards exit.
pub fn handle_close_request(
    mut requests: MessageReader<WindowCloseRequested>,
    mut state: ResMut<EditorState>,
    mut exit: MessageWriter<AppExit>,
) {
    if requests.is_empty() {
        return;
    }
    requests.clear();

    if state.exit_dialog_open {
        return;
    }

    if state.dirty {
        state.exit_dialog_open = true;
    } else {
        exit.write(AppExit::Success);
    }
}

/// Handles the Save/Discard/Cancel buttons on the unsaved-changes exit
/// dialog. Save only quits once the board actually validates and writes
/// successfully — otherwise the error surfaces via `state.message` and the
/// dialog stays open so the user can fix the board.
pub fn handle_exit_dialog_buttons(
    mut state: ResMut<EditorState>,
    board: Res<EditorBoard>,
    mut exit: MessageWriter<AppExit>,
    interaction_q: Query<(&ExitDialogButton, &Interaction), Changed<Interaction>>,
) {
    for (kind, interaction) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match kind {
            ExitDialogButton::Save => {
                if save_board(&board, &mut state) {
                    state.exit_dialog_open = false;
                    exit.write(AppExit::Success);
                }
            }
            ExitDialogButton::Discard => {
                state.exit_dialog_open = false;
                exit.write(AppExit::Success);
            }
            ExitDialogButton::Cancel => {
                state.exit_dialog_open = false;
            }
        }
    }
}

/// Handles keyboard input for the filename text box.
/// Tab focuses/confirms; Escape cancels; printable chars and Backspace edit.
pub fn handle_text_input(
    mut key_events: MessageReader<KeyboardInput>,
    mut state: ResMut<EditorState>,
) {
    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if state.filename_editing {
            match &event.logical_key {
                Key::Character(c) => {
                    let s = c.as_str();
                    // Ignore control characters (e.g. Ctrl+C → '\x03')
                    if s.chars().all(|ch| !ch.is_control()) {
                        state.filename.push_str(s);
                    }
                }
                Key::Space => state.filename.push(' '),
                Key::Backspace => {
                    state.filename.pop();
                }
                Key::Enter | Key::Tab => {
                    state.filename_editing = false;
                }
                Key::Escape => {
                    state.filename = state.filename_backup.clone();
                    state.filename_editing = false;
                }
                _ => {}
            }
        } else if event.logical_key == Key::Tab {
            state.filename_backup = state.filename.clone();
            state.filename_editing = true;
        }
    }
}

/// Focuses the save-path box when it is clicked. Tab-to-focus is handled by
/// `handle_text_input`.
pub fn handle_filename_click(
    mut state: ResMut<EditorState>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<FilenameInputBox>)>,
) {
    if state.exit_dialog_open {
        return;
    }

    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed && !state.filename_editing {
            state.filename_backup = state.filename.clone();
            state.filename_editing = true;
        }
    }
}

pub fn handle_mouse(
    mouse_btn: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    plane_q: Query<&GlobalTransform, With<GroundPlane>>,
    mut board: ResMut<EditorBoard>,
    state: Res<EditorState>,
) {
    // Don't let clicks through the modal exit-confirmation overlay paint the
    // board underneath it.
    if state.exit_dialog_open {
        return;
    }

    let left = mouse_btn.pressed(MouseButton::Left);
    let right = mouse_btn.pressed(MouseButton::Right);
    if !left && !right {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };
    let Ok(plane_gt) = plane_q.single() else {
        return;
    };

    // Skip clicks that land on the right-side UI panel (220 px wide).
    if let Some(cursor) = window.cursor_position()
        && cursor.x > window.width() - 220.0
    {
        return;
    }

    let Some((cell, _hit)) = raycast_to_grid_cell(
        window,
        camera,
        cam_transform,
        plane_gt,
        board.width as u32,
        board.height as u32,
    ) else {
        return;
    };

    let tool = if right {
        EditorTile::Empty
    } else {
        state.current_tool
    };

    board.set_tile(cell.y as usize, cell.x as usize, tool);
}

/// Editor-local counterpart to `crate::camera::pan_camera` — the editor has
/// no `GameStateResource`/`ConfigResource`, so it can't share that system
/// directly; it gathers board size from `EditorBoard`/`CELL_SIZE` instead and
/// calls the same shared clamp helper. Same Shift+Left-click drag scheme as
/// the live game.
pub fn pan_camera(
    mut query: Query<(&mut Transform, &Projection), With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    board: Res<EditorBoard>,
    mut pan_state: ResMut<PanState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let dragging = shift_held(&keyboard_input) && mouse_button.pressed(MouseButton::Left);
    let delta = drag_delta(&mut pan_state, dragging, window.cursor_position());

    let board_size = Vec2::new(board.width as f32 * CELL_SIZE, board.height as f32 * CELL_SIZE);
    let window_size = Vec2::new(window.width(), window.height());
    let padding = CELL_SIZE * PAN_PADDING_CELLS;

    for (mut transform, projection) in query.iter_mut() {
        let Projection::Orthographic(ortho) = projection else {
            continue;
        };
        // X is not negated here — see the matching note in
        // `camera::pan_camera` (this camera's NEG_Z up-vector mirrors the X
        // axis on screen relative to world X).
        let direction = Vec3::new(delta.x * ortho.scale, 0.0, delta.y * ortho.scale);
        pan_and_clamp(
            &mut transform.translation,
            direction,
            ortho.scale,
            window_size,
            board_size,
            padding,
        );
    }
}

pub fn tick_message_timer(time: Res<Time>, mut state: ResMut<EditorState>) {
    if state.message_timer > 0.0 {
        state.message_timer -= time.delta_secs();
        if state.message_timer <= 0.0 {
            state.message = None;
            state.message_timer = 0.0;
        }
    }
}
