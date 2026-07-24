use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::gen_stub_pyfunction;

use gg_bevy::editor::EditorBoard;

/// Launch the interactive world editor.
///
/// Parameters
/// ----------
/// load:
///     Path to an existing world YAML file to edit. Mutually exclusive with `size`.
/// size:
///     Dimensions of a new blank board in "WxH" format (e.g. "20x15"). Mutually
///     exclusive with `load`.
/// out:
///     Where to save when pressing S. Falls back to `load`, then a generated
///     default name.
#[gen_stub_pyfunction]
#[pyfunction(name = "run_editor")]
#[pyo3(signature=(load=None, size=None, out=None))]
pub fn run_editor(load: Option<&str>, size: Option<&str>, out: Option<&str>) -> PyResult<()> {
    let board = match (load, size) {
        (Some(_), Some(_)) => {
            return Err(PyTypeError::new_err("`load` and `size` are mutually exclusive"));
        }
        (Some(path), None) => {
            EditorBoard::from_file(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        }
        (None, Some(dims)) => {
            let (w_str, h_str) = dims
                .split_once('x')
                .ok_or_else(|| PyTypeError::new_err("`size` must be formatted as WIDTHxHEIGHT"))?;
            let width: usize = w_str
                .trim()
                .parse()
                .map_err(|_| PyTypeError::new_err("Invalid width in `size`"))?;
            let height: usize = h_str
                .trim()
                .parse()
                .map_err(|_| PyTypeError::new_err("Invalid height in `size`"))?;
            EditorBoard::new_empty(width, height)
        }
        (None, None) => {
            return Err(PyTypeError::new_err("One of `load` or `size` must be provided"));
        }
    };

    gg_bevy::build_editor_app(board, out.map(String::from), load.map(String::from)).run();

    Ok(())
}
