use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyclass_complex_enum;

#[gen_stub_pyclass_complex_enum]
#[pyclass(name = "EntityType")]
#[derive(Debug, Clone)]
pub enum EntityType {
    Empty(),
    Wall(),
    Goblet(i32),
    Agent(),
    Ghost(),
}
