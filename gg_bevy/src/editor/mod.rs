pub mod board;
pub mod input;
pub mod ui;

use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;
use gg_core::{Goblet, WorldFile};

use crate::camera::{zoom_in, zoom_out};

/// Cell size used purely for the editor's own rendering scale (the editor
/// isn't tied to a `GGConfig`, unlike the live game).
pub const CELL_SIZE: f32 = 5.0;

/// Width of the right-side tool panel, shared by the UI, camera, and input.
pub const PANEL_WIDTH: f32 = 240.0;

// ─── Tile type ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum EditorTile {
    #[default]
    Empty,
    Wall,
    Agent,
    Ghost,
    Goblet(i32),
}

// ─── Board resource ───────────────────────────────────────────────────────────

#[derive(Resource, Clone)]
pub struct EditorBoard {
    pub tiles: Vec<Vec<EditorTile>>,
    pub width: usize,
    pub height: usize,
}

impl EditorBoard {
    pub fn new_empty(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![vec![EditorTile::Empty; width]; height],
            width,
            height,
        }
    }

    /// Load from a `gg_core::WorldFile` YAML world.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let board = WorldFile::from_file(path)
            .and_then(|world| world.into_board())
            .map_err(|e| e.to_string())?;
        let (width, height) = (board.width, board.height);
        let mut tiles = vec![vec![EditorTile::Empty; width]; height];

        for &(col, row) in &board.wall_positions {
            tiles[row][col] = EditorTile::Wall;
        }
        for Goblet { position, reward } in &board.goblets {
            tiles[position.1][position.0] = EditorTile::Goblet(*reward);
        }
        let (ac, ar) = board.agent_position;
        tiles[ar][ac] = EditorTile::Agent;
        if let Some((gc, gr)) = board.ghost_position {
            tiles[gr][gc] = EditorTile::Ghost;
        }

        Ok(Self { tiles, width, height })
    }

    /// Paint a tile, enforcing the single-agent / at-most-one-ghost constraint.
    pub fn set_tile(&mut self, row: usize, col: usize, tile: EditorTile) {
        if tile == EditorTile::Agent {
            for r in &mut self.tiles {
                for t in r.iter_mut() {
                    if *t == EditorTile::Agent {
                        *t = EditorTile::Empty;
                    }
                }
            }
        } else if tile == EditorTile::Ghost {
            for r in &mut self.tiles {
                for t in r.iter_mut() {
                    if *t == EditorTile::Ghost {
                        *t = EditorTile::Empty;
                    }
                }
            }
        }
        self.tiles[row][col] = tile;
    }

    pub fn agent_count(&self) -> usize {
        self.tiles
            .iter()
            .flatten()
            .filter(|&&t| t == EditorTile::Agent)
            .count()
    }

    pub fn ghost_count(&self) -> usize {
        self.tiles
            .iter()
            .flatten()
            .filter(|&&t| t == EditorTile::Ghost)
            .count()
    }

    /// Validates and converts to a `gg_core::WorldFile`.
    pub fn to_world_file(&self) -> Result<WorldFile, String> {
        if self.agent_count() != 1 {
            return Err(format!(
                "Need exactly 1 agent (found {})",
                self.agent_count()
            ));
        }
        if self.ghost_count() > 1 {
            return Err(format!(
                "At most 1 ghost allowed (found {})",
                self.ghost_count()
            ));
        }

        let mut walls = Vec::new();
        let mut goblets = Vec::new();
        let mut agent_position = None;
        let mut ghost_position = None;

        for (row, cols) in self.tiles.iter().enumerate() {
            for (col, tile) in cols.iter().enumerate() {
                match tile {
                    EditorTile::Wall => walls.push((col, row)),
                    EditorTile::Agent => agent_position = Some((col, row)),
                    EditorTile::Ghost => ghost_position = Some((col, row)),
                    EditorTile::Goblet(reward) => goblets.push(Goblet {
                        position: (col, row),
                        reward: *reward,
                    }),
                    EditorTile::Empty => {}
                }
            }
        }

        Ok(WorldFile {
            width: self.width,
            height: self.height,
            walls,
            agent_position: agent_position.expect("validated above"),
            ghost_position,
            goblets,
        })
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        self.to_world_file()?.to_file(path).map_err(|e| e.to_string())
    }
}

// ─── Editor state resource ────────────────────────────────────────────────────

#[derive(Resource)]
pub struct EditorState {
    pub current_tool: EditorTile,
    /// Reward assigned to newly-painted goblets; adjustable with `[`/`]`.
    pub current_goblet_reward: i32,
    pub show_grid: bool,
    pub out_path: Option<String>,
    pub load_path: Option<String>,
    pub message: Option<String>,
    pub message_timer: f32,
    /// The save path shown/edited in the filename text box.
    pub filename: String,
    /// Whether the filename text box has keyboard focus.
    pub filename_editing: bool,
    /// Snapshot of `filename` when editing began (for Escape-to-cancel).
    pub filename_backup: String,
    /// Whether the board has unsaved edits since the last load/save (the
    /// very first tile-sync after load/new doesn't count — see `mark_dirty`).
    pub dirty: bool,
    /// Whether the "unsaved changes" exit-confirmation dialog is showing.
    /// While open, all other keyboard/mouse input is suppressed.
    pub exit_dialog_open: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            current_tool: EditorTile::Wall,
            current_goblet_reward: 1,
            show_grid: true,
            out_path: None,
            load_path: None,
            message: None,
            message_timer: 0.0,
            filename: String::new(),
            filename_editing: false,
            filename_backup: String::new(),
            dirty: false,
            exit_dialog_open: false,
        }
    }
}

impl EditorState {
    pub fn show_message(&mut self, msg: impl Into<String>, duration_secs: f32) {
        self.message = Some(msg.into());
        self.message_timer = duration_secs;
    }
}

/// Shortens absolute paths under the current working directory for display in
/// the editor's filename box. Relative and external paths are left unchanged.
fn display_path(path: String) -> String {
    let path = std::path::Path::new(&path);
    if !path.is_absolute() {
        return path.to_string_lossy().into_owned();
    }

    std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Attempts to save `board` to `state.filename` (or a generated default
/// name), updating `state`'s message/dirty/out_path accordingly. Returns
/// whether the save succeeded — used to gate whether the exit-confirmation
/// dialog's Save button is allowed to actually quit (an invalid board stays
/// open so the user can fix it, with the error surfacing via the message).
pub fn save_board(board: &EditorBoard, state: &mut EditorState) -> bool {
    let path = if state.filename.is_empty() {
        format!("world_{}x{}.yaml", board.width, board.height)
    } else {
        state.filename.clone()
    };
    match board.save_to_file(&path) {
        Ok(()) => {
            state.out_path = Some(path.clone());
            state.filename = path;
            state.dirty = false;
            state.show_message(format!("Saved: {}", state.filename), 2.5);
            true
        }
        Err(e) => {
            state.show_message(format!("Cannot save: {e}"), 4.0);
            false
        }
    }
}

/// Marks the board dirty on real edits. Skips the very first invocation
/// (gated on `resource_changed::<EditorBoard>` in the plugin, which is also
/// true the instant the board resource is inserted) so loading an existing
/// file or the initial tile spawn doesn't immediately count as an edit.
pub fn mark_dirty(mut state: ResMut<EditorState>, mut has_run: Local<bool>) {
    if !*has_run {
        *has_run = true;
        return;
    }
    state.dirty = true;
}

// ─── Plugin ───────────────────────────────────────────────────────────────────

pub struct EditorPlugin {
    pub board: EditorBoard,
    pub out_path: Option<String>,
    pub load_path: Option<String>,
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let filename = self
            .out_path
            .clone()
            .or_else(|| self.load_path.clone())
            .map(display_path)
            .unwrap_or_default();
        let state = EditorState {
            out_path: self.out_path.clone(),
            load_path: self.load_path.clone(),
            filename,
            ..Default::default()
        };
        app.insert_resource(self.board.clone());
        app.insert_resource(state);
        app.init_resource::<crate::camera::PanState>();

        app.add_systems(PreStartup, board::init_visual_assets);
        app.add_systems(
            Startup,
            (
                board::setup_scene,
                board::setup_grid,
                board::setup_tiles,
                ui::setup_ui,
            ),
        );
        app.add_systems(
            Update,
            (
                input::handle_keyboard,
                input::handle_close_request,
                input::handle_text_input,
                input::handle_filename_click,
                input::handle_mouse,
                input::handle_exit_dialog_buttons,
                input::tick_message_timer,
                board::sync_grid_visibility,
                board::update_goblet_label_positions,
                ui::sync_ui,
                ui::sync_dirty_indicator,
                ui::sync_exit_dialog,
                zoom_in.run_if(input_pressed(KeyCode::Equal)),
                zoom_out.run_if(input_pressed(KeyCode::Minus)),
                input::pan_camera,
                crate::camera::update_pan_cursor,
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                board::sync_tiles,
                board::sync_goblet_labels,
                mark_dirty,
            )
                .run_if(resource_changed::<EditorBoard>),
        );
    }
}
