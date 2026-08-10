//! Thin `Resource` newtypes wrapping `gg_core` types for the ECS.

use bevy::prelude::*;
use gg_core::{GGConfig, GameState, Policy, ValueGrid};

#[derive(Resource, Clone)]
pub struct ConfigResource(pub GGConfig);

#[derive(Resource, Clone)]
pub struct GameStateResource(pub GameState);

#[derive(Resource, Clone)]
pub struct PolicyResource(pub Policy);

/// The learned value function to visualize as a heatmap, if one was
/// provided (older checkpoints / hand-built policies may not have one).
#[derive(Resource, Clone)]
pub struct HeatmapResource(pub Option<ValueGrid>);

#[derive(Resource)]
pub struct PolicyTimer(pub Timer);

/// Speed multiplier for automatic policy-driven simulation.
#[derive(Resource)]
pub struct SimulationSpeed {
    index: usize,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self { index: 3 }
    }
}

impl SimulationSpeed {
    const MULTIPLIERS: [f32; 8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 4.0];

    pub fn multiplier(&self) -> f32 {
        Self::MULTIPLIERS[self.index]
    }

    pub fn slower(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    pub fn faster(&mut self) {
        self.index = (self.index + 1).min(Self::MULTIPLIERS.len() - 1);
    }
}

/// Whether the automatic policy-driven simulation is paused. Only
/// meaningful (and only toggleable) when `GhostPolicy::Teleop` is not
/// active — in Teleop mode the simulation only ever advances on a keypress,
/// so "pause/play" doesn't apply and Space keeps its Teleop meaning
/// ("ghost stays in place") instead.
#[derive(Resource, Default)]
pub struct SimulationPaused(pub bool);

/// One-frame latch set when Space restarts a finished episode so other
/// Space-driven systems (teleop stay / pause toggle) ignore that same keypress.
#[derive(Resource, Default)]
pub struct RestartedThisFrame(pub bool);
