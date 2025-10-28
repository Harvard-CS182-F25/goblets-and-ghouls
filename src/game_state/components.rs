use crate::{
    agent::{Action, GhostPolicy},
    core::GGConfig,
};
use bevy::prelude::*;
use bevy_prng::WyRand;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_complex_enum, gen_stub_pymethods};
use rand::{
    Rng, SeedableRng,
    seq::{IndexedRandom, IteratorRandom},
};
use std::collections::HashSet;

#[derive(Component)]
pub struct HoverBox;

#[derive(Component)]
pub struct HoverBoxText;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct HoverCell {
    pub cell: Option<UVec2>, // (col, row)
    pub world_hit: Option<Vec3>,
}

#[gen_stub_pyclass_complex_enum]
#[pyclass(name = "EntityType")]
#[derive(Debug, Clone, Component, Reflect)]
pub enum EntityType {
    Empty(),
    Wall(),
    Goblet(i32),
    Agent(),
    Ghost(),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Agent {
    Player,
    Ghost,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct VisualizePolicy(pub bool);

#[gen_stub_pyclass]
#[pyclass(name = "GameState")]
#[derive(Debug, Clone, Resource)]
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
            rng: WyRand::default(),
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

    pub fn transition(&mut self, action: Action) -> Self {
        if self.done {
            return self.clone();
        }

        let board = self
            .board
            .transition(&mut self.rng, action, self.active_player, &self.config);

        let state = GameState::from(board)
            .with_initial_board(&self.initial_board)
            .with_config(&self.config);

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

    pub fn with_config(mut self, config: &GGConfig) -> Self {
        self.config = config.clone();

        if let Some(seed) = config.episode_seed {
            self.with_seed(seed as u64);
        }

        self
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Goblet")]
#[derive(Debug, Clone)]
pub struct Goblet {
    #[pyo3(get)]
    pub position: (usize, usize),
    #[pyo3(get)]
    pub reward: i32,
}

#[gen_stub_pyclass]
#[pyclass(name = "Board")]
#[derive(Debug, Clone, Component)]
pub struct Board {
    #[pyo3(get)]
    pub agent_position: (usize, usize),
    #[pyo3(get)]
    pub ghost_position: Option<(usize, usize)>,
    #[pyo3(get)]
    pub goblets: Vec<Goblet>,
    #[pyo3(get)]
    pub wall_positions: HashSet<(usize, usize)>,
    #[pyo3(get)]
    pub width: usize,
    #[pyo3(get)]
    pub height: usize,
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
            agent_position,
            ghost_position,
            goblets,
            wall_positions,
            width,
            height,
        }
    }

    pub fn transition(
        &self,
        rng: &mut impl Rng,
        action: Action,
        active_player: Agent,
        config: &GGConfig,
    ) -> Board {
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

    pub fn transition_det(&self, action: Action, active_player: Agent) -> Self {
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

                    board.ghost_position = Some(new_position);
                }
            }
        };

        board
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
