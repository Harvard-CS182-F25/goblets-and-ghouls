mod systems;

use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;

pub use systems::*;

/// Extra slack (world units) let past the point where the board's edge
/// reaches the viewport edge, so panning to an edge doesn't leave the board
/// flush against the window border. Expressed as a multiple of cell size so
/// it scales with the world rather than being a fixed absolute margin.
pub const PAN_PADDING_CELLS: f32 = 2.0;

const FIT_MARGIN: f32 = 1.15;
const ZOOM_OUT_MARGIN: f32 = 3.0;
const MAX_CELL_SIZE_PX: f32 = 500.0;

/// Board-relative bounds for the orthographic camera scale.
#[derive(Resource)]
pub struct CameraZoomLimits {
    pub min_scale: f32,
    pub max_scale: f32,
}

impl Default for CameraZoomLimits {
    fn default() -> Self {
        Self {
            min_scale: 0.0,
            max_scale: f32::INFINITY,
        }
    }
}

impl CameraZoomLimits {
    pub fn new(board_size: Vec2, cell_size: f32, viewport_size: Vec2, initial_scale: f32) -> Self {
        let initial_scale = initial_scale.abs();
        Self {
            // Preserve a deliberately closer initial view, but prevent further
            // zooming once a cell reaches the practical display-size limit.
            min_scale: (cell_size / MAX_CELL_SIZE_PX).min(initial_scale),
            max_scale: (fit_scale(board_size, viewport_size) * ZOOM_OUT_MARGIN).max(initial_scale),
        }
    }
}

/// Orthographic scale that fits a board inside a viewport with a little
/// breathing room around its edges.
pub fn fit_scale(board_size: Vec2, viewport_size: Vec2) -> f32 {
    let viewport_size = viewport_size.max(Vec2::ONE);
    (board_size.x * FIT_MARGIN / viewport_size.x).max(board_size.y * FIT_MARGIN / viewport_size.y)
}

/// Tracks the cursor position from the previous frame while a Shift+Left-
/// click drag is in progress, so panning systems can compute a per-frame
/// screen-space delta. `None` whenever a drag isn't currently active.
#[derive(Resource, Default)]
pub struct PanState {
    pub last_cursor: Option<Vec2>,
}

/// Moves `translation` by `direction` (unit vector in the X/Z ground plane)
/// and clamps the result so the board never scrolls more than `padding`
/// world units past the viewport edge, at the camera's current zoom. `center`
/// is the camera's resting position, which offsets the editor board left of
/// its right-side panel.
///
/// Pure, resource-free helper shared by the live game's camera (which reads
/// board size from `GameStateResource`/`ConfigResource`) and the world
/// editor's camera (which has neither resource and reads board size from
/// `EditorBoard` instead) — mirrors how `segmentation`'s `pan_and_clamp`
/// is shared between its game view and its own editor.
pub fn pan_and_clamp(
    translation: &mut Vec3,
    direction: Vec3,
    scale: f32,
    window_size: Vec2,
    board_size: Vec2,
    padding: f32,
    center: Vec2,
) {
    *translation += direction;

    let half_view = window_size * scale.abs() / 2.0;
    let half_board = board_size / 2.0;
    let max = (half_board - half_view + Vec2::splat(padding)).max(Vec2::ZERO);

    translation.x = translation.x.clamp(center.x - max.x, center.x + max.x);
    translation.z = translation.z.clamp(center.y - max.y, center.y + max.y);
}

/// Whether either Shift key is currently held.
pub fn shift_held(keyboard_input: &ButtonInput<KeyCode>) -> bool {
    keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
}

/// Computes the per-frame screen-space cursor delta for a Shift+Left-click
/// drag, advancing `pan_state` for the next frame. Returns `Vec2::ZERO`
/// whenever a drag isn't in progress (not `dragging`), or on the first
/// frame of a new drag (no prior cursor position yet to diff against).
pub fn drag_delta(pan_state: &mut PanState, dragging: bool, cursor: Option<Vec2>) -> Vec2 {
    let delta = if dragging {
        match (pan_state.last_cursor, cursor) {
            (Some(last), Some(current)) => current - last,
            _ => Vec2::ZERO,
        }
    } else {
        Vec2::ZERO
    };
    pan_state.last_cursor = if dragging { cursor } else { None };
    delta
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PanState>();
        app.init_resource::<CameraZoomLimits>();
        app.add_systems(Startup, systems::setup_camera);
        app.add_systems(
            Update,
            (
                systems::zoom_in.run_if(input_pressed(KeyCode::Equal)),
                systems::zoom_out.run_if(input_pressed(KeyCode::Minus)),
                systems::pan_camera,
                systems::update_pan_cursor,
            ),
        );
    }
}
