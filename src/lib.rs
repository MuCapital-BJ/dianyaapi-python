mod py_types;
mod transcribe_stream;
mod transcribe_wrapper;
mod types;

use crate::transcribe_stream::TranscribeStream;
use crate::transcribe_wrapper::TranscribeApi;
use pyo3::{
    Bound, PyResult, Python, pymodule,
    types::{PyModule, PyModuleMethods},
};

#[pymodule]
pub fn dianyaapi(_: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TranscribeApi>()?;
    m.add_class::<TranscribeStream>()?;
    Ok(())
}
