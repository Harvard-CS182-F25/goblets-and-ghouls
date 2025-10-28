use bevy::prelude::*;
use derivative::Derivative;
use pyo3::{prelude::*, types::PyTuple};
use pyo3_stub_gen::derive::{gen_stub_pyclass_enum, gen_stub_pymethods};

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Agent;

#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect, Default)]
#[reflect(Component)]
pub struct GhostAgent;

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
#[gen_stub_pyclass_enum]
#[pyclass(name = "Action", module = "gg_core._core")]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Action::Up => "Up",
            Action::Down => "Down",
            Action::Left => "Left",
            Action::Right => "Right",
        };
        write!(f, "{}", s)
    }
}

impl From<u8> for Action {
    fn from(tag: u8) -> Self {
        match tag {
            0 => Action::Up,
            1 => Action::Down,
            2 => Action::Left,
            3 => Action::Right,
            _ => panic!("Invalid Action tag: {}", tag),
        }
    }
}

impl From<Action> for u8 {
    fn from(action: Action) -> Self {
        match action {
            Action::Up => 0,
            Action::Down => 1,
            Action::Left => 2,
            Action::Right => 3,
        }
    }
}

impl From<&Action> for u8 {
    fn from(action: &Action) -> Self {
        match action {
            Action::Up => 0,
            Action::Down => 1,
            Action::Left => 2,
            Action::Right => 3,
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Action {
    #[staticmethod]
    fn from_int(tag: u8) -> PyResult<Self> {
        Ok(Action::from(tag))
    }

    fn __reduce__(&self, py: Python) -> PyResult<(Py<PyAny>, Py<PyAny>)> {
        let module = py.import("gg_core._core")?;
        let cls = module.getattr("Action")?;
        let from_int = cls.getattr("from_int")?;
        let tag: u8 = self.into();

        let args = PyTuple::new(py, [tag])?;
        Ok((from_int.into(), args.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Message)]
pub struct PlayerActionMessage {
    pub action: Action,
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
