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
