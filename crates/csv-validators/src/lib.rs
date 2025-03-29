use std::sync::Arc;
use pyo3::prelude::*;
use pyo3::types::{PyModule};
use csv_validator_core::{ValidatorSpec, ValidationOptions, validate_file,ValidationIssue};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::FromPyObject;


#[pyclass]
#[derive(Clone)]
pub struct PyValidatorSpec {
    inner: ValidatorSpec,
}

#[pymethods]
impl PyValidatorSpec {
    #[staticmethod]
    pub fn illegal_chars(chars: Vec<String>) -> Self {
        Self {
            inner: ValidatorSpec::IllegalChars {
                illegal_chars: chars,
                enabled: true,
            },
        }
    }

    #[staticmethod]
    pub fn field_count(expected: usize) -> Self {
        Self {
            inner: ValidatorSpec::FieldCount {
                expected,
                enabled: true,
            },
        }
    }

    #[staticmethod]
    pub fn line_length(max_length: usize) -> Self {
        Self {
            inner: ValidatorSpec::LineLength {
                max_length,
                enabled: true,
            },
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyValidationOptions {
    #[pyo3(get, set)]
    pub threads: usize,
    #[pyo3(get, set)]
    pub batch_size: usize,
    #[pyo3(get, set)]
    pub buffer_size: usize,
}

#[pymethods]
impl PyValidationOptions {
    #[new]
    fn new() -> Self {
        Self {
            threads: num_cpus::get(),
            batch_size: 100_000,
            buffer_size: 8 * 1024 * 1024,
        }
    }
}
impl Default for PyValidationOptions {
    fn default() -> Self {
        Self {
            threads: num_cpus::get(),
            batch_size: 100_000,
            buffer_size: 8 * 1024 * 1024,
        }
    }
}


impl From<&PyValidationOptions> for ValidationOptions {
    fn from(py: &PyValidationOptions) -> Self {
        Self {
            threads: py.threads,
            batch_size: py.batch_size,
            buffer_size: py.buffer_size,
        }
    }
}

#[pyclass]
pub struct PyValidationIssue {
    #[pyo3(get)]
    pub validator: String,
    #[pyo3(get)]
    pub line_number: usize,
    #[pyo3(get)]
    pub position: Option<usize>,
    #[pyo3(get)]
    pub message: String,
}

impl From<ValidationIssue> for PyValidationIssue {
    fn from(issue: ValidationIssue) -> Self {
        Self {
            validator: issue.validator.parse().unwrap(),
            line_number: issue.line_number,
            position: issue.position,
            message: issue.message,
        }
    }
}

#[pyfunction]
pub fn validate_file_py(
    py: Python<'_>,
    path: &str,
    validators: Vec<PyValidatorSpec>,
    options: Option<PyObject>,
) -> PyResult<Vec<PyValidationIssue>> {
    let opts: PyValidationOptions = match options {
        Some(obj) => obj.extract(py)?,
        None => PyValidationOptions::default(),
    };

    let specs: Vec<_> = validators
        .into_iter()
        .map(|v| v.inner.into_validator(b',')) // TODO: make separator configurable
        .collect();

    let issues = validate_file(path, Arc::new(specs), (&opts).into())?;
    Ok(issues.into_iter().map(Into::into).collect())
}

// #[pyfunction]
// pub fn validate_lines_py(
//     lines: &PyList,
//     validators: Vec<PyValidatorSpec>,
//     options: Option<PyValidationOptions>,
// ) -> PyResult<Vec<PyValidationIssue>> {
//     let lines: Vec<(usize, Vec<u8>)> = lines
//         .iter()
//         .enumerate()
//         .map(|(i, line)| {
//             let s: String = line.extract().unwrap_or_default();
//             (i + 1, s.into_bytes())
//         })
//         .collect();
//
//     let specs: Vec<_> = validators
//         .into_iter()
//         .map(|v| v.inner.into_validator(b',')) // TODO: support separator
//         .collect();
//
//     let opts = options.as_ref().unwrap_or(&PyValidationOptions::default());
//     let issues = csv_validator_core::validate_lines(lines, Arc::new(specs), opts.into());
//     Ok(issues.into_iter().map(Into::into).collect())
// }


#[pyfunction]
pub fn jahallo(py: Python) -> PyResult<String>{
    Ok("jahallo".to_string())
}

// main entrypoint for python module
#[pymodule]
fn csv_validators(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(jahallo, m)?)?;
    m.add_class::<PyValidatorSpec>()?;
    m.add_class::<PyValidationIssue>()?;
    m.add_class::<PyValidationOptions>()?;
    m.add_function(wrap_pyfunction!(validate_file_py, m)?)?;
    Ok(())
}