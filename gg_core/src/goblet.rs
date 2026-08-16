use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

#[gen_stub_pyclass]
#[pyclass(name = "Goblet")]
#[derive(Debug, Clone, Serialize, Deserialize)]

/// Represents the state of a single goblet.
pub struct Goblet {
    #[pyo3(get)]
    pub position: (usize, usize),
    #[pyo3(get)]
    pub reward: i32,
}
