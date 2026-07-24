use crate::action::Action;

/// A grid of values over the joint (agent position, ghost position) space,
/// stored as a flat, row-major array of length `width*height*width*height`.
///
/// Ghost-free grids (e.g. Value Iteration's policy/value output) are
/// represented via [`Grid::from_agent_grid`], which broadcasts a
/// `(width, height)` grid of values across every ghost slice so `get`/`set`
/// behave uniformly regardless of how the grid was built.
#[derive(Debug, Clone)]
pub struct Grid<T> {
    values: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl<T: Clone> Grid<T> {
    pub fn new(width: usize, height: usize, default: T) -> Self {
        Self {
            values: vec![default; width * height * width * height],
            width,
            height,
        }
    }

    /// Broadcast a ghost-independent `(width, height)` grid (row-major,
    /// `agent.1 * width + agent.0`) across every ghost slice.
    pub fn from_agent_grid(agent_values: Vec<T>, width: usize, height: usize) -> Self {
        debug_assert_eq!(agent_values.len(), width * height);

        let ghost_slots = width * height;
        let mut values = Vec::with_capacity(agent_values.len() * ghost_slots);
        for value in &agent_values {
            values.extend(std::iter::repeat_n(value.clone(), ghost_slots));
        }

        Self {
            values,
            width,
            height,
        }
    }

    fn index(&self, agent: (usize, usize), ghost: (usize, usize)) -> usize {
        let agent_idx = agent.1 * self.width + agent.0;
        let ghost_idx = ghost.1 * self.width + ghost.0;
        agent_idx * (self.width * self.height) + ghost_idx
    }

    pub fn get(&self, agent: (usize, usize), ghost: (usize, usize)) -> T {
        self.values[self.index(agent, ghost)].clone()
    }

    pub fn set(&mut self, agent: (usize, usize), ghost: (usize, usize), value: T) {
        let idx = self.index(agent, ghost);
        self.values[idx] = value;
    }
}

/// A ghost-position-dependent policy: `pi[agent, ghost] -> Action`.
pub type Policy = Grid<Action>;

/// A ghost-position-dependent value function: `V[agent, ghost] -> f32`
/// (e.g. Value Iteration's `V`, or `max_a Q(agent, ghost, a)` for Q-learning).
pub type ValueGrid = Grid<f32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_round_trip() {
        let mut policy = Policy::new(3, 3, Action::Up);
        policy.set((1, 2), (0, 1), Action::Right);
        assert_eq!(policy.get((1, 2), (0, 1)), Action::Right);
        // Untouched entries stay at the default.
        assert_eq!(policy.get((0, 0), (0, 0)), Action::Up);
        assert_eq!(policy.get((1, 2), (0, 0)), Action::Up);
    }

    #[test]
    fn distinct_agent_ghost_pairs_map_to_distinct_indices() {
        let width = 4;
        let height = 4;
        let mut policy = Policy::new(width, height, Action::Up);

        for ax in 0..width {
            for ay in 0..height {
                for gx in 0..width {
                    for gy in 0..height {
                        let action = match (ax + ay + gx + gy) % 4 {
                            0 => Action::Up,
                            1 => Action::Down,
                            2 => Action::Left,
                            _ => Action::Right,
                        };
                        policy.set((ax, ay), (gx, gy), action);
                    }
                }
            }
        }

        for ax in 0..width {
            for ay in 0..height {
                for gx in 0..width {
                    for gy in 0..height {
                        let expected = match (ax + ay + gx + gy) % 4 {
                            0 => Action::Up,
                            1 => Action::Down,
                            2 => Action::Left,
                            _ => Action::Right,
                        };
                        assert_eq!(policy.get((ax, ay), (gx, gy)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn from_agent_grid_broadcasts_across_ghost_positions() {
        let width = 2;
        let height = 2;
        // row-major: agent.1 * width + agent.0
        let agent_actions = vec![Action::Up, Action::Down, Action::Left, Action::Right];
        let policy = Policy::from_agent_grid(agent_actions, width, height);

        for ax in 0..width {
            for ay in 0..height {
                let expected = match ay * width + ax {
                    0 => Action::Up,
                    1 => Action::Down,
                    2 => Action::Left,
                    _ => Action::Right,
                };
                for gx in 0..width {
                    for gy in 0..height {
                        assert_eq!(policy.get((ax, ay), (gx, gy)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn value_grid_get_set_round_trip() {
        let mut values = ValueGrid::new(3, 3, 0.0_f32);
        values.set((1, 2), (0, 1), 4.5);
        assert_eq!(values.get((1, 2), (0, 1)), 4.5);
        assert_eq!(values.get((0, 0), (0, 0)), 0.0);
    }

    #[test]
    fn value_grid_from_agent_grid_broadcasts_across_ghost_positions() {
        let width = 2;
        let height = 2;
        let agent_values = vec![1.0_f32, 2.0, 3.0, 4.0];
        let values = ValueGrid::from_agent_grid(agent_values, width, height);

        for ax in 0..width {
            for ay in 0..height {
                let expected = (ay * width + ax) as f32 + 1.0;
                for gx in 0..width {
                    for gy in 0..height {
                        assert_eq!(values.get((ax, ay), (gx, gy)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn value_grid_distinct_agent_ghost_pairs_map_to_distinct_indices() {
        let width = 3;
        let height = 3;
        let mut values = ValueGrid::new(width, height, -1.0_f32);

        for ax in 0..width {
            for ay in 0..height {
                for gx in 0..width {
                    for gy in 0..height {
                        let v = (ax + ay * 10 + gx * 100 + gy * 1000) as f32;
                        values.set((ax, ay), (gx, gy), v);
                    }
                }
            }
        }

        for ax in 0..width {
            for ay in 0..height {
                for gx in 0..width {
                    for gy in 0..height {
                        let expected = (ax + ay * 10 + gx * 100 + gy * 1000) as f32;
                        assert_eq!(values.get((ax, ay), (gx, gy)), expected);
                    }
                }
            }
        }
    }
}
