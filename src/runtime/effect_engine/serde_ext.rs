//! Serde extensions for pyo3

use pyo3::prelude::*;
use pyo3::types::*;

/// Convert the given serde_json::Value to a Python object
///
/// # Parameters
///
/// * `value`: value to convert
pub fn to_object(value: serde_json::Value, py: Python) -> PyObject {
    use serde_json::Value;

    match value {
        Value::Null => py.None(),
        Value::Bool(val) => val.into_py(py),
        Value::Number(num) => {
            if num.is_i64() {
                num.as_i64().unwrap().into_py(py)
            } else if num.is_u64() {
                num.as_u64().unwrap().into_py(py)
            } else {
                num.as_f64().unwrap().into_py(py)
            }
        }
        Value::String(string) => string.into_py(py),
        Value::Array(values) => {
            PyList::new(py, values.into_iter().map(|item| to_object(item, py))).to_object(py)
        }
        Value::Object(map) => {
            let dict = PyDict::new(py);

            for (k, v) in map.into_iter() {
                if let Err(_error) = dict.set_item(&k, to_object(v, py)) {
                    warn!("failed to convert Python effect args (key: '{}')", k);
                }
            }

            dict.to_object(py)
        }
    }
}
