use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::{
    Rng,
    seq::{IndexedRandom, IteratorRandom},
};
use std::collections::HashSet;

use crate::agent::Agent;
use crate::config::GGConfig;
use crate::entity_type::EntityType;
use crate::goblet::Goblet;

#[gen_stub_pyclass]
#[pyclass(name = "Board")]
#[derive(Debug, Clone)]

/// Represents the state of the board.
/// Stores the dimensions and board layout.
pub struct Board {
    #[pyo3(get)]
    pub width: usize,
    #[pyo3(get)]
    pub height: usize,
    #[pyo3(get)]
    pub agent_position: (usize, usize),
    /// Returns the true position of the ghost, regardless of visibility.
    /// If there is no ghost, returns None.
    #[pyo3(get)]
    pub ghost_position: Option<(usize, usize)>,
    #[pyo3(get)]
    pub goblets: Vec<Goblet>,
    #[pyo3(get)]
    pub wall_positions: HashSet<(usize, usize)>,
}

fn get_circle_indices(
    center: (usize, usize),
    radius: usize,
    width: usize,
    height: usize,
) -> Vec<(usize, usize)> {
    let mut indices = Vec::new();
    let (cx, cy) = center;
    let r_sq = (radius * radius) as isize;

    for y in cy.saturating_sub(radius - 1)..(cy + radius).min(height - 1) {
        for x in cx.saturating_sub(radius - 1)..(cx + radius).min(width - 1) {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;
            if dx * dx + dy * dy <= r_sq {
                indices.push((x, y));
            }
        }
    }

    indices
}

impl Board {
    pub fn new(rng: &mut impl Rng, config: &GGConfig) -> Self {
        let width = (config.world_generation.world_width / config.world_generation.cell_size)
            .round() as usize;
        let height = (config.world_generation.world_height / config.world_generation.cell_size)
            .round() as usize;
        let num_obstacles = config.world_generation.num_obstacles;

        let free_positions = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .collect::<Vec<_>>();

        let wall_positions = free_positions
            .iter()
            .cloned()
            .choose_multiple(rng, num_obstacles)
            .iter()
            .flat_map(|&pos| {
                get_circle_indices(
                    pos,
                    rng.random_range(1..=config.world_generation.obstacle_radius_cells),
                    width,
                    height,
                )
            })
            .chain((0..width).flat_map(|x| vec![(x, 0), (x, height - 1)]))
            .chain((0..height).flat_map(|y| vec![(0, y), (width - 1, y)]))
            .collect::<HashSet<_>>();

        let free_positions = free_positions
            .into_iter()
            .filter(|pos| !wall_positions.contains(pos))
            .collect::<Vec<_>>();

        let agent_position = free_positions
            .choose(rng)
            .cloned()
            .expect("No free positions available");

        let free_positions = free_positions
            .into_iter()
            .filter(|&pos| pos != agent_position)
            .collect::<Vec<_>>();

        let ghost_position = if config.agent.ghost_policy.is_some() {
            Some(
                free_positions
                    .choose(rng)
                    .cloned()
                    .expect("No free positions available for ghost"),
            )
        } else {
            None
        };

        let free_positions = free_positions
            .into_iter()
            .filter(|&pos| Some(pos) != ghost_position)
            .collect::<Vec<_>>();

        let goblets = (0..config.goblets.number)
            .filter_map(|_| {
                free_positions.choose(rng).cloned().map(|position| Goblet {
                    position,
                    reward: rng.random_range(
                        -(config.goblets.max_reward as i32)..=(config.goblets.max_reward as i32),
                    ),
                })
            })
            .collect::<Vec<_>>();

        Self {
            width,
            height,
            agent_position,
            ghost_position,
            goblets,
            wall_positions,
        }
    }

    pub fn transition(
        &self,
        rng: &mut impl Rng,
        action: crate::action::Action,
        active_player: Agent,
        config: &GGConfig,
    ) -> Board {
        use crate::action::Action;

        const ACTIONS: [Action; 4] = [Action::Up, Action::Right, Action::Down, Action::Left];
        let rotated_actions = match action {
            Action::Up => ACTIONS,
            Action::Right => [ACTIONS[1], ACTIONS[2], ACTIONS[3], ACTIONS[0]],
            Action::Down => [ACTIONS[2], ACTIONS[3], ACTIONS[0], ACTIONS[1]],
            Action::Left => [ACTIONS[3], ACTIONS[0], ACTIONS[1], ACTIONS[2]],
        };

        let weights = match active_player {
            Agent::Player => config.agent.transition,
            Agent::Ghost => [1.0, 0.0, 0.0, 0.0],
        };

        let enumerated_actions: Vec<(usize, &Action)> =
            rotated_actions.iter().enumerate().collect::<Vec<_>>();
        let (_, chosen_action) = enumerated_actions
            .choose_weighted(rng, |&(idx, _)| weights[idx])
            .expect("Should have at least one movement option");

        self.transition_det(**chosen_action, active_player)
    }

    pub fn transition_det(&self, action: crate::action::Action, active_player: Agent) -> Self {
        use crate::action::Action;

        let (dx, dy) = match action {
            Action::Up => (0, -1),
            Action::Right => (1, 0),
            Action::Down => (0, 1),
            Action::Left => (-1, 0),
        };

        let mut board = self.clone();
        match active_player {
            Agent::Player => {
                let new_x = board
                    .agent_position
                    .0
                    .saturating_add_signed(dx)
                    .clamp(0, board.width - 1);

                let new_y = board
                    .agent_position
                    .1
                    .saturating_add_signed(dy)
                    .clamp(0, board.height - 1);

                let new_position = (new_x, new_y);
                if !board.wall_positions.contains(&new_position) {
                    board.agent_position = new_position;
                }
            }
            Agent::Ghost => {
                if let Some(ghost_pos) = board.ghost_position {
                    let new_x: usize = ghost_pos
                        .0
                        .saturating_add_signed(dx)
                        .clamp(0, board.width - 1);

                    let new_y = ghost_pos
                        .1
                        .saturating_add_signed(dy)
                        .clamp(0, board.height - 1);
                    let new_position = (new_x, new_y);

                    if !board.wall_positions.contains(&new_position) {
                        board.ghost_position = Some(new_position);
                    }
                }
            }
        };

        board
    }

    /// Whether `to` is visible from `from`, i.e. no wall lies on the line
    /// between them.
    pub fn has_line_of_sight(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        crate::los::has_line_of_sight(from, to, &self.wall_positions)
    }

    /// The ghost position directly observed from `agent_pos`, if any. With
    /// occlusion enabled, a ghost hidden behind a wall is hidden; invalid or
    /// hypothetical states where the ghost is inside a wall are also treated
    /// as hidden.
    pub fn observed_ghost_position_for(
        &self,
        agent_pos: (usize, usize),
        occlusion_enabled: bool,
    ) -> Option<(usize, usize)> {
        let ghost = self.ghost_position?;
        if occlusion_enabled
            && (self.wall_positions.contains(&ghost) || !self.has_line_of_sight(agent_pos, ghost))
        {
            None
        } else {
            Some(ghost)
        }
    }

    /// The ghost position a policy should be indexed with, as observed from
    /// `agent_pos`: the ghost's real position if one exists and (when
    /// `occlusion_enabled`) is visible from `agent_pos`, otherwise `agent_pos`
    /// itself. This lets `pi[x,y,x,y]` stand in for "no ghost" / "ghost
    /// occluded" so a policy always fits neatly into a dense matrix.
    pub fn effective_ghost_position_for(
        &self,
        agent_pos: (usize, usize),
        occlusion_enabled: bool,
    ) -> (usize, usize) {
        self.observed_ghost_position_for(agent_pos, occlusion_enabled)
            .unwrap_or(agent_pos)
    }

    /// [`Board::effective_ghost_position_for`] observed from the actual
    /// current agent position.
    pub fn effective_ghost_position(&self, occlusion_enabled: bool) -> (usize, usize) {
        self.effective_ghost_position_for(self.agent_position, occlusion_enabled)
    }

    pub fn get(&self, position: &(usize, usize)) -> EntityType {
        if self.wall_positions.contains(position) {
            EntityType::Wall()
        } else if let Some((_, reward)) = self
            .goblets
            .iter()
            .map(|g| (g.position, g.reward))
            .find(|(pos, _)| pos == position)
        {
            EntityType::Goblet(reward)
        } else if self.agent_position == *position {
            EntityType::Agent()
        } else if self.ghost_position == Some(*position) {
            EntityType::Ghost()
        } else {
            EntityType::Empty()
        }
    }
}

#[gen_stub_pymethods]
impl Board {
    fn __getitem__(&self, position: (usize, usize)) -> EntityType {
        self.get(&position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board_with(
        agent_position: (usize, usize),
        ghost_position: Option<(usize, usize)>,
        wall_positions: HashSet<(usize, usize)>,
    ) -> Board {
        Board {
            agent_position,
            ghost_position,
            goblets: Vec::new(),
            wall_positions,
            width: 5,
            height: 5,
        }
    }

    #[test]
    fn occluded_ghost_falls_back_to_agent_position_only_when_enabled() {
        let mut walls = HashSet::new();
        walls.insert((2, 0));
        let board = board_with((0, 0), Some((4, 0)), walls);

        assert_eq!(board.effective_ghost_position(false), (4, 0));
        assert_eq!(board.effective_ghost_position(true), (0, 0));
    }

    #[test]
    fn ghost_inside_wall_is_hidden_only_when_occlusion_is_enabled() {
        let mut walls = HashSet::new();
        walls.insert((4, 0));
        let board = board_with((0, 0), Some((4, 0)), walls);

        assert_eq!(board.effective_ghost_position(false), (4, 0));
        assert_eq!(board.effective_ghost_position(true), (0, 0));
        assert_eq!(board.observed_ghost_position_for((0, 0), false), Some((4, 0)));
        assert_eq!(board.observed_ghost_position_for((0, 0), true), None);
    }

    #[test]
    fn effective_ghost_position_for_supports_hypothetical_agent_cells() {
        let mut walls = HashSet::new();
        walls.insert((2, 0));
        let board = board_with((0, 0), Some((4, 0)), walls);

        // From (0,0) the ghost at (4,0) is occluded by the wall at (2,0)...
        assert_eq!(board.effective_ghost_position_for((0, 0), true), (0, 0));
        // ...but from (0,1) the wall at (2,0) doesn't block the diagonal line.
        assert_eq!(board.effective_ghost_position_for((0, 4), true), (4, 0));
    }
}
