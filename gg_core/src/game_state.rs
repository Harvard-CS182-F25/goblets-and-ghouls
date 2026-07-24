use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use rand::{SeedableRng, seq::IndexedRandom};
use wyrand::WyRand;

use crate::action::Action;
use crate::agent::Agent;
use crate::board::Board;
use crate::config::{GGConfig, GhostPolicy};

#[gen_stub_pyclass]
#[pyclass(name = "GameState")]
#[derive(Debug, Clone)]
pub struct GameState {
    #[pyo3(get)]
    pub board: Board,
    #[pyo3(get)]
    pub reward: i32,
    #[pyo3(get)]
    pub done: bool,
    pub active_player: Agent,

    pub initial_board: Box<Board>,
    pub rng: WyRand,
    pub rng_seed: u64,
    pub config: GGConfig,
}

#[gen_stub_pymethods]
#[pymethods]
impl GameState {
    fn all_states(&self) -> Vec<GameState> {
        let (width, height) = (self.board.width, self.board.height);
        let mut states = Vec::new();

        for x in 0..width {
            for y in 0..height {
                let mut new_board = self.board.clone();
                new_board.agent_position = (x, y);
                let new_state = GameState::from(new_board);
                states.push(new_state);
            }
        }

        if self.board.ghost_position.is_some() {
            for x in 0..width {
                for y in 0..height {
                    let mut new_board = self.board.clone();
                    new_board.ghost_position = Some((x, y));
                    let new_state = GameState::from(new_board);
                    states.push(new_state);
                }
            }
        }

        states
    }

    fn next_state(&self, action: Action) -> GameState {
        let board = self.board.transition_det(action, Agent::Player);
        GameState::from(board)
    }

    /// The ghost's position as currently observed by the agent: `None` if
    /// there's no ghost, or if one exists but isn't currently visible (e.g.
    /// behind a wall, when `ghost_occlusion` is enabled); otherwise its real
    /// position.
    #[getter]
    fn ghost_position(&self) -> Option<(usize, usize)> {
        let ghost = self.board.ghost_position?;
        if self.config.agent.ghost_occlusion
            && !self
                .board
                .has_line_of_sight(self.board.agent_position, ghost)
        {
            return None;
        }
        Some(ghost)
    }

    pub fn with_seed(&self, seed: u64) -> GameState {
        let mut new_state = self.clone();
        new_state.rng_seed = seed;
        new_state.rng = WyRand::from_seed(seed.to_ne_bytes());
        new_state
    }

    pub fn step(&mut self, action: Action) -> GameState {
        let state = self.transition(action);
        assert_eq!(state.active_player, Agent::Player);
        state
    }

    fn reset(&self) -> (GameState, u64) {
        let state = GameState::from((*self.initial_board).clone())
            .with_initial_board(&self.initial_board)
            .with_config(&self.config);

        let seed = if let Some(seed) = self.config.episode_seed {
            seed as u64
        } else {
            rand::random::<u32>().into()
        };

        let state = state.with_seed(seed);

        (state, seed)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("GameState({})", self.__str__()?))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "GameState(agent_position={:?}, ghost_position={:?}, reward={}, done={})",
            self.board.agent_position, self.board.ghost_position, self.reward, self.done
        ))
    }
}

impl From<Board> for GameState {
    fn from(board: Board) -> Self {
        let reward = if let Some(ghost_position) = board.ghost_position
            && board.agent_position == ghost_position
        {
            i32::MIN
        } else if let Some((_, value)) = board
            .goblets
            .iter()
            .map(|g| (g.position, g.reward))
            .find(|(pos, _)| *pos == board.agent_position)
        {
            value
        } else {
            0
        };

        let done = reward != 0;

        let seed = rand::random::<u32>();

        Self {
            board: board.clone(),
            reward,
            done,
            active_player: Agent::Player,
            initial_board: Box::new(board),
            rng: WyRand::from_rng(&mut rand::rng()),
            rng_seed: seed.into(),
            config: GGConfig::default(),
        }
    }
}

impl GameState {
    pub fn with_initial_board(mut self, board: &Board) -> Self {
        self.initial_board = Box::new(board.clone());
        self
    }

    /// The player-move half of a transition, shared by `transition` (which
    /// follows it with an automatically-decided ghost move) and
    /// `step_teleop` (which follows it with an externally-supplied one).
    fn player_transition(&mut self, action: Action) -> Self {
        if self.done {
            return self.clone();
        }

        let board = self
            .board
            .transition(&mut self.rng, action, self.active_player, &self.config);

        GameState::from(board)
            .with_initial_board(&self.initial_board)
            .with_config(&self.config)
    }

    pub fn transition(&mut self, action: Action) -> Self {
        let state = self.player_transition(action);

        if state.done {
            return state;
        }

        let ghost_action = match self.config.agent.ghost_policy {
            Some(GhostPolicy::Random) => {
                let actions = [Action::Up, Action::Right, Action::Down, Action::Left];
                *actions
                    .choose(&mut self.rng)
                    .expect("Should have at least one action")
            }
            Some(GhostPolicy::Chaser) => {
                let ghost_pos = self
                    .board
                    .ghost_position
                    .expect("Ghost position should be present");
                let agent_pos = self.board.agent_position;

                let dx = agent_pos.0 as isize - ghost_pos.0 as isize;
                let dy = agent_pos.1 as isize - ghost_pos.1 as isize;

                if dx.abs() > dy.abs() {
                    if dx > 0 { Action::Right } else { Action::Left }
                } else if dy > 0 {
                    Action::Down
                } else {
                    Action::Up
                }
            }
            Some(GhostPolicy::Teleop) => panic!(
                "GhostPolicy::Teleop requires GameState::step_teleop() — the ghost's move must be supplied externally, so step()/next_state()/transition() cannot be used with this ghost policy"
            ),
            None => {
                assert!(self.board.ghost_position.is_none());
                return state;
            }
        };

        let board = state.board.transition_det(ghost_action, Agent::Ghost);

        GameState::from(board)
            .with_initial_board(&self.initial_board)
            .with_config(&self.config)
    }

    /// Like `step`, but for a `GhostPolicy::Teleop` ghost: `ghost_direction`
    /// is the human-supplied direction for the ghost this tick (`None`
    /// means the ghost stays in place — e.g. Space was pressed instead of a
    /// direction key — rather than some automatic policy deciding).
    pub fn step_teleop(&mut self, action: Action, ghost_direction: Option<Action>) -> GameState {
        assert_eq!(
            self.config.agent.ghost_policy,
            Some(GhostPolicy::Teleop),
            "step_teleop requires GhostPolicy::Teleop"
        );

        let state = self.player_transition(action);
        if state.done {
            return state;
        }

        match ghost_direction {
            Some(direction) => {
                let board = state.board.transition_det(direction, Agent::Ghost);
                GameState::from(board)
                    .with_initial_board(&self.initial_board)
                    .with_config(&self.config)
            }
            None => state,
        }
    }

    pub fn with_config(mut self, config: &GGConfig) -> Self {
        self.config = config.clone();

        if let Some(seed) = config.episode_seed {
            self.with_seed(seed as u64);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AgentConfig;
    use std::collections::HashSet;

    fn teleop_state(agent_position: (usize, usize), ghost_position: (usize, usize)) -> GameState {
        let board = Board {
            agent_position,
            ghost_position: Some(ghost_position),
            goblets: Vec::new(),
            wall_positions: HashSet::new(),
            width: 5,
            height: 5,
        };
        let config = GGConfig {
            agent: AgentConfig {
                ghost_policy: Some(GhostPolicy::Teleop),
                transition: [1.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
            ..Default::default()
        };
        GameState::from(board).with_config(&config)
    }

    #[test]
    fn step_teleop_moves_ghost_in_supplied_direction() {
        let mut state = teleop_state((0, 0), (4, 4));
        let next = state.step_teleop(Action::Right, Some(Action::Left));
        assert_eq!(next.board.agent_position, (1, 0));
        assert_eq!(next.board.ghost_position, Some((3, 4)));
    }

    #[test]
    fn step_teleop_none_keeps_ghost_in_place() {
        let mut state = teleop_state((0, 0), (4, 4));
        let next = state.step_teleop(Action::Right, None);
        assert_eq!(next.board.agent_position, (1, 0));
        assert_eq!(next.board.ghost_position, Some((4, 4)));
    }

    #[test]
    #[should_panic(expected = "step_teleop requires GhostPolicy::Teleop")]
    fn step_teleop_panics_if_ghost_policy_is_not_teleop() {
        let board = Board {
            agent_position: (0, 0),
            ghost_position: Some((4, 4)),
            goblets: Vec::new(),
            wall_positions: HashSet::new(),
            width: 5,
            height: 5,
        };
        let config = GGConfig {
            agent: AgentConfig {
                ghost_policy: Some(GhostPolicy::Chaser),
                transition: [1.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
            ..Default::default()
        };
        let mut state = GameState::from(board).with_config(&config);
        state.step_teleop(Action::Right, Some(Action::Left));
    }

    #[test]
    #[should_panic(expected = "GhostPolicy::Teleop requires GameState::step_teleop()")]
    fn transition_panics_if_ghost_policy_is_teleop() {
        let mut state = teleop_state((0, 0), (4, 4));
        state.transition(Action::Right);
    }
}
