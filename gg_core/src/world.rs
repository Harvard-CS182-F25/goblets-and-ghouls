//! Custom world file format: a structured YAML description of a fixed board
//! layout (walls, agent/ghost start positions, goblets with rewards), used as
//! an alternative to procedural generation (`Board::new`).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::board::Board;
use crate::goblet::Goblet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFile {
    pub width: usize,
    pub height: usize,
    #[serde(default)]
    pub walls: Vec<(usize, usize)>,
    pub agent_position: (usize, usize),
    #[serde(default)]
    pub ghost_position: Option<(usize, usize)>,
    #[serde(default)]
    pub goblets: Vec<Goblet>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseWorldError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse world file: {0}")]
    Deserialize(#[from] serde_yaml::Error),
    #[error("position {0:?} is out of bounds for a {1}x{2} board")]
    OutOfBounds((usize, usize), usize, usize),
    #[error("position {0:?} is a wall and cannot host {1}")]
    PositionIsWall((usize, usize), &'static str),
    #[error("multiple entities occupy position {0:?}")]
    DuplicatePosition((usize, usize)),
}

impl WorldFile {
    pub fn from_file(path: &str) -> Result<Self, ParseWorldError> {
        let contents = std::fs::read_to_string(path)?;
        let world: WorldFile = serde_yaml::from_str(&contents)?;
        Ok(world)
    }

    pub fn to_file(&self, path: &str) -> Result<(), ParseWorldError> {
        let contents = serde_yaml::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Validates and converts into a `Board`: the agent position must be
    /// in-bounds and not a wall; the ghost position (if any) must be
    /// in-bounds, not a wall, and distinct from the agent; every goblet must
    /// be in-bounds, not on a wall, and at a unique position.
    pub fn into_board(self) -> Result<Board, ParseWorldError> {
        let in_bounds = |pos: (usize, usize)| pos.0 < self.width && pos.1 < self.height;

        if !in_bounds(self.agent_position) {
            return Err(ParseWorldError::OutOfBounds(
                self.agent_position,
                self.width,
                self.height,
            ));
        }

        let wall_positions: HashSet<(usize, usize)> = self.walls.iter().copied().collect();

        if wall_positions.contains(&self.agent_position) {
            return Err(ParseWorldError::PositionIsWall(
                self.agent_position,
                "the agent",
            ));
        }

        let mut occupied = HashSet::new();
        occupied.insert(self.agent_position);

        if let Some(ghost_position) = self.ghost_position {
            if !in_bounds(ghost_position) {
                return Err(ParseWorldError::OutOfBounds(
                    ghost_position,
                    self.width,
                    self.height,
                ));
            }
            if wall_positions.contains(&ghost_position) {
                return Err(ParseWorldError::PositionIsWall(ghost_position, "the ghost"));
            }
            if !occupied.insert(ghost_position) {
                return Err(ParseWorldError::DuplicatePosition(ghost_position));
            }
        }

        for goblet in &self.goblets {
            if !in_bounds(goblet.position) {
                return Err(ParseWorldError::OutOfBounds(
                    goblet.position,
                    self.width,
                    self.height,
                ));
            }
            if wall_positions.contains(&goblet.position) {
                return Err(ParseWorldError::PositionIsWall(goblet.position, "a goblet"));
            }
            if !occupied.insert(goblet.position) {
                return Err(ParseWorldError::DuplicatePosition(goblet.position));
            }
        }

        Ok(Board {
            agent_position: self.agent_position,
            ghost_position: self.ghost_position,
            goblets: self.goblets,
            wall_positions,
            width: self.width,
            height: self.height,
        })
    }

    /// Inverse of `into_board` — used by the editor's save.
    pub fn from_board(board: &Board) -> Self {
        Self {
            width: board.width,
            height: board.height,
            walls: board.wall_positions.iter().copied().collect(),
            agent_position: board.agent_position,
            ghost_position: board.ghost_position,
            goblets: board.goblets.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixtures live at the workspace-root `worlds/` dir, not inside this
    /// crate, so build an absolute path independent of the test binary's cwd
    /// (`cargo test` runs with cwd = the crate's own manifest directory).
    fn fixture(name: &str) -> String {
        format!("{}/../worlds/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn valid_world_loads_and_converts() {
        let world = WorldFile::from_file(&fixture("simple.yaml")).expect("should parse");
        let board = world.into_board().expect("should validate");

        assert_eq!(board.width, 5);
        assert_eq!(board.height, 5);
        assert_eq!(board.agent_position, (0, 4));
        assert_eq!(board.ghost_position, Some((4, 0)));
        assert_eq!(board.goblets.len(), 1);
        assert!(board.wall_positions.contains(&(2, 2)));
    }

    #[test]
    fn agent_on_wall_is_rejected() {
        let world = WorldFile::from_file(&fixture("agent_on_wall.yaml")).expect("should parse");
        let err = world.into_board().expect_err("should reject");
        assert!(matches!(err, ParseWorldError::PositionIsWall(_, "the agent")));
    }

    #[test]
    fn out_of_bounds_goblet_is_rejected() {
        let world =
            WorldFile::from_file(&fixture("goblet_out_of_bounds.yaml")).expect("should parse");
        let err = world.into_board().expect_err("should reject");
        assert!(matches!(err, ParseWorldError::OutOfBounds(..)));
    }

    #[test]
    fn duplicate_position_is_rejected() {
        let world =
            WorldFile::from_file(&fixture("duplicate_position.yaml")).expect("should parse");
        let err = world.into_board().expect_err("should reject");
        assert!(matches!(err, ParseWorldError::DuplicatePosition(_)));
    }

    #[test]
    fn missing_file_is_io_error() {
        let err = WorldFile::from_file(&fixture("does_not_exist.yaml")).expect_err("should fail");
        assert!(matches!(err, ParseWorldError::Io(_)));
    }

    #[test]
    fn round_trip_through_from_board() {
        let world = WorldFile::from_file(&fixture("simple.yaml")).expect("should parse");
        let board = world.into_board().expect("should validate");
        let round_tripped = WorldFile::from_board(&board).into_board().expect("should validate");

        assert_eq!(round_tripped.width, board.width);
        assert_eq!(round_tripped.height, board.height);
        assert_eq!(round_tripped.agent_position, board.agent_position);
        assert_eq!(round_tripped.ghost_position, board.ghost_position);
        assert_eq!(round_tripped.wall_positions, board.wall_positions);
    }
}
