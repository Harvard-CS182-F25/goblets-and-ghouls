use bevy::prelude::*;
use derivative::Derivative;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass_enum;

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Agent;

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct GhostAgent;

#[derive(Debug, Clone, Copy, PartialEq)]
#[gen_stub_pyclass_enum]
#[pyclass(name = "Action")]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Message)]
pub struct ActionMessage {
    pub action: Action,
    pub entity: Entity,
}

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
