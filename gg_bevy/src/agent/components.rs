use bevy::prelude::*;
use derivative::Derivative;
use gg_core::Action;

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Agent;

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct GhostAgent;

/// How the ghost should move for the step this `PlayerActionMessage`
/// triggers. `Auto` (the default, non-Teleop ghost policies) lets
/// `GameState::step` decide automatically; `Teleop*` variants carry a
/// human-supplied direction (or "stay in place") for `GhostPolicy::Teleop`,
/// consumed via `GameState::step_teleop` instead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostInput {
    Auto,
    TeleopMove(Action),
    TeleopStay,
}

#[derive(Debug, Clone, PartialEq, Message)]
pub struct PlayerActionMessage {
    pub action: Action,
    pub ghost_input: GhostInput,
}

/// Marks the pause/play HUD badge's background and text, so
/// `update_pause_indicator` can recolor/relabel it each time
/// `SimulationPaused` changes. Not spawned at all in Teleop mode (see
/// `spawn_pause_indicator`) — pause/play isn't a meaningful concept there.
#[derive(Component)]
pub struct PauseIndicatorBadge;

#[derive(Component)]
pub struct PauseIndicatorText;

/// Marks the automatic simulation-speed HUD text.
#[derive(Component)]
pub struct SimulationSpeedText;

#[derive(Debug, Clone, Bundle, Derivative)]
#[derivative(Default)]
pub struct AgentBundle {
    #[derivative(Default(value = "Name::new(\"Agent\")"))]
    pub name: Name,
    pub agent: Agent,
    pub position: Transform,
}

impl AgentBundle {
    pub fn new(name: &str, position: Vec3) -> Self {
        Self {
            name: Name::new(name.to_string()),
            agent: Agent,
            position: Transform::from_translation(position),
        }
    }
}

#[derive(Debug, Clone, Bundle, Derivative)]
#[derivative(Default)]
pub struct GhostAgentBundle {
    #[derivative(Default(value = "Name::new(\"GhostAgent\")"))]
    pub name: Name,
    pub agent: GhostAgent,
    pub position: Transform,
}

impl GhostAgentBundle {
    pub fn new(name: &str, position: Vec3) -> Self {
        Self {
            name: Name::new(name.to_string()),
            agent: GhostAgent,
            position: Transform::from_translation(position),
        }
    }
}
