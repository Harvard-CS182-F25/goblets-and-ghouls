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

/// Represents the game state and holds the bulk of the game logic.
pub struct GameState {
    /// Returns the state of the board.
    #[pyo3(get)]
    pub board: Board,
    /// Represents the immediate reward of the current state. Equals the
    /// configured ghost penalty if the agent has been caught by the ghost,
    /// the value of a goblet if the agent has found a goblet, and zero
    /// otherwise.
    #[pyo3(get)]
    pub reward: i32,
    /// Returns True if the agent has collected a goblet or has been caught
    /// by the ghost.
    #[pyo3(get)]
    pub done: bool,
    pub active_player: Agent,

    pub initial_board: Box<Board>,
    pub rng: WyRand,
    pub rng_seed: u64,
    pub config: GGConfig,
}

impl GameState {
    fn caught_reward(config: &GGConfig) -> i32 {
        config.agent.ghost_penalty
    }

    fn outcome_for(board: &Board, config: &GGConfig) -> (i32, bool) {
        let caught = board.ghost_position == Some(board.agent_position);
        let goblet_reward = board
            .goblets
            .iter()
            .map(|g| (g.position, g.reward))
            .find(|(pos, _)| *pos == board.agent_position)
            .map(|(_, value)| value);

        let reward = if caught {
            Self::caught_reward(config)
        } else if let Some(value) = goblet_reward {
            value
        } else {
            0
        };

        (reward, caught || goblet_reward.is_some())
    }

    fn refresh_outcome(mut self) -> Self {
        (self.reward, self.done) = Self::outcome_for(&self.board, &self.config);
        self
    }

    fn with_runtime_from(mut self, other: &GameState) -> Self {
        self.initial_board = other.initial_board.clone();
        self.rng = other.rng.clone();
        self.rng_seed = other.rng_seed;
        self.config = other.config.clone();
        self.refresh_outcome()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GameState {
    /// Returns the current position of the agent.
    #[getter]
    fn agent_position(&self) -> (usize, usize) {
        self.board.agent_position
    }

    /// Returns the position of the ghost as observed by the agent. If there is
    /// no ghost, or if the ghost isn't currently visible (e.g., behind a wall)
    /// when `ghost_occlusion` is enabled, returns None. Invalid states (e.g.,
    /// where the ghost is inside a wall) are also treated as hidden.
    #[getter(observed_ghost_position)]
    fn observed_ghost_position(&self) -> Option<(usize, usize)> {
        self.board.observed_ghost_position_for(
            self.board.agent_position,
            self.config.agent.ghost_occlusion,
        )
    }

    /// Returns the positions of goblets visible from the agent's current
    /// position. Goblets hidden behind walls are omitted.
    #[getter]
    fn visible_goblet_positions(&self) -> Vec<(usize, usize)> {
        self.board
            .goblets
            .iter()
            .filter(|goblet| {
                self.board
                    .has_line_of_sight(self.board.agent_position, goblet.position)
            })
            .map(|goblet| goblet.position)
            .collect()
    }

    /// Returns the positions of walls visible from the agent's current
    /// position. A wall is visible if no other wall lies between the agent
    /// and that wall.
    #[getter]
    fn visible_wall_positions(&self) -> Vec<(usize, usize)> {
        let mut visible_wall_positions = self
            .board
            .wall_positions
            .iter()
            .copied()
            .filter(|&wall| self.board.has_line_of_sight(self.board.agent_position, wall))
            .collect::<Vec<_>>();
        visible_wall_positions.sort_unstable();
        visible_wall_positions
    }

    /// Returns the episode seed driving this state's RNG.
    #[getter]
    fn episode_seed(&self) -> u64 {
        self.rng_seed
    }
    
    /// Returns a list of `width * height` game states where the agent's
    /// position is varied across all cells. Note that invalid states are not
    /// filtered out.
    fn all_states(&self) -> Vec<GameState> {
        let (width, height) = (self.board.width, self.board.height);
        let mut states = Vec::with_capacity(width * height);

        for x in 0..width {
            for y in 0..height {
                let mut new_board = self.board.clone();
                new_board.agent_position = (x, y);
                states.push(GameState::from(new_board).with_runtime_from(self));
            }
        }

        states
    }

    /// Returns the game state that would result from the agent moving in the
    /// given direction. This move is deterministic, and does not update the
    /// ghost's position.
    fn next_state(&self, action: Action) -> GameState {
        let board = self.board.transition_det(action, Agent::Player);
        GameState::from(board).with_runtime_from(self)
    }

    /// Returns a new game state resulting from one full environment step.
    /// This step is not deterministic, and updates both the agent and the
    /// ghost's position using the episode seed. Fails for the Teleop ghost.
    pub fn step(&mut self, action: Action) -> GameState {
        let state = self.transition(action);
        assert_eq!(state.active_player, Agent::Player);
        state
    }

    /// Returns a copy of the current game state with a fixed episode seed,
    /// used to determine the actions taken by the agent and the ghost.
    pub fn with_seed(&self, seed: u64) -> GameState {
        let mut new_state = self.clone();
        new_state.rng_seed = seed;
        new_state.rng = WyRand::from_seed(seed.to_ne_bytes());
        new_state
    }

    /// Returns a new game state set to the original configuration, along with
    /// the episode seed it uses. When `seed` is None, a fresh seed is sampled.
    #[pyo3(signature = (seed=None))]
    pub fn reset(&self, seed: Option<u64>) -> (GameState, u64) {
        let state = GameState::from((*self.initial_board).clone())
            .with_initial_board(&self.initial_board)
            .with_config(&self.config);

        let seed = seed.unwrap_or_else(|| rand::random::<u32>().into());

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
        let seed = rand::random::<u32>();
        let config = GGConfig::default();
        let (reward, done) = Self::outcome_for(&board, &config);

        Self {
            board: board.clone(),
            reward,
            done,
            active_player: Agent::Player,
            initial_board: Box::new(board),
            rng: WyRand::from_seed(u64::from(seed).to_ne_bytes()),
            rng_seed: seed.into(),
            config,
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

        GameState::from(board).with_runtime_from(self)
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

        GameState::from(board).with_runtime_from(self)
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
                GameState::from(board).with_runtime_from(&state)
            }
            None => state,
        }
    }

    pub fn with_config(mut self, config: &GGConfig) -> Self {
        self.config = config.clone();
        self.refresh_outcome()
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
    fn caught_reward_uses_configured_ghost_penalty() {
        let board = Board {
            agent_position: (2, 2),
            ghost_position: Some((2, 2)),
            goblets: Vec::new(),
            wall_positions: HashSet::new(),
            width: 5,
            height: 5,
        };
        let config = GGConfig {
            agent: AgentConfig {
                ghost_penalty: -70,
                ..Default::default()
            },
            ..Default::default()
        };

        let state = GameState::from(board).with_config(&config);
        assert_eq!(state.reward, -70);
        assert!(state.done);
    }

    #[test]
    fn zero_reward_goblet_is_still_terminal() {
        let board = Board {
            agent_position: (2, 2),
            ghost_position: None,
            goblets: vec![crate::goblet::Goblet {
                position: (2, 2),
                reward: 0,
            }],
            wall_positions: HashSet::new(),
            width: 5,
            height: 5,
        };

        let state = GameState::from(board);
        assert_eq!(state.reward, 0);
        assert!(state.done);
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

    fn state_signature(state: &GameState) -> ((usize, usize), Option<(usize, usize)>, i32, bool) {
        (
            state.board.agent_position,
            state.board.ghost_position,
            state.reward,
            state.done,
        )
    }

    #[test]
    fn seeded_state_remains_deterministic_across_multiple_steps() {
        let board = Board {
            agent_position: (2, 2),
            ghost_position: Some((4, 4)),
            goblets: Vec::new(),
            wall_positions: HashSet::new(),
            width: 6,
            height: 6,
        };
        let config = GGConfig {
            agent: AgentConfig {
                ghost_policy: Some(GhostPolicy::Random),
                transition: [0.4, 0.3, 0.2, 0.1],
                ..Default::default()
            },
            episode_seed: Some(1234),
            ..Default::default()
        };

        let initial_state = GameState::from(board).with_config(&config).with_seed(1234);
        let mut left = initial_state.clone();
        let mut right = initial_state;

        for action in [Action::Up, Action::Right, Action::Down, Action::Left] {
            let next_left = left.step(action);
            let next_right = right.step(action);
            assert_eq!(state_signature(&next_left), state_signature(&next_right));
            left = next_left;
            right = next_right;
        }
    }

    #[test]
    fn reset_uses_explicit_seed_instead_of_config_seed() {
        let board = Board {
            agent_position: (2, 2),
            ghost_position: Some((4, 4)),
            goblets: Vec::new(),
            wall_positions: HashSet::new(),
            width: 6,
            height: 6,
        };
        let config = GGConfig {
            episode_seed: Some(1234),
            ..Default::default()
        };

        let state = GameState::from(board).with_config(&config);
        let (reset_state, reset_seed) = state.reset(Some(5678));

        assert_eq!(reset_seed, 5678);
        assert_eq!(reset_state.rng_seed, 5678);
    }

    #[test]
    fn all_states_varies_only_agent_position() {
        let board = Board {
            agent_position: (1, 1),
            ghost_position: Some((4, 4)),
            goblets: Vec::new(),
            wall_positions: HashSet::new(),
            width: 3,
            height: 2,
        };
        let state = GameState::from(board).with_seed(1234);

        let states = state.all_states();

        assert_eq!(states.len(), 6);
        assert!(states.iter().all(|s| s.board.ghost_position == Some((4, 4))));
        assert!(states.iter().all(|s| s.rng_seed == 1234));
    }
}
