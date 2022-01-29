use std::{convert::TryFrom, path::Path};

use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
    types::{PyByteArray, PyTuple},
};
use pythonize::pythonize;

use crate::{
    effects::{RuntimeMethodError, RuntimeMethods},
    image::RawImage,
    models::Color,
};

mod context;
use context::Context;

pub type Error = pyo3::PyErr;

#[derive(Default, Debug, Clone, Copy)]
pub struct PythonProvider;

impl PythonProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::Provider for PythonProvider {
    fn supports(&self, script_path: &str) -> bool {
        script_path.ends_with(".py")
    }

    fn run(
        &self,
        full_script_path: &Path,
        args: serde_json::Value,
        methods: crate::effects::instance::InstanceMethods,
    ) -> Result<(), super::ProviderError> {
        Ok(do_run(methods, args, |py| {
            // Run script
            py.run(
                std::fs::read_to_string(&full_script_path)?.as_str(),
                None,
                None,
            )?;

            Ok(())
        })?)
    }
}

impl From<RuntimeMethodError> for PyErr {
    fn from(value: RuntimeMethodError) -> PyErr {
        match value {
            RuntimeMethodError::InvalidArguments { .. } => PyTypeError::new_err(value.to_string()),
            _ => PyRuntimeError::new_err(value.to_string()),
        }
    }
}

/// Check if the effect should abort execution
#[pyfunction]
fn abort() -> bool {
    Context::with_current(|m| m.abort())
}

/// Set a new color for the leds
#[pyfunction(args = "*")]
#[pyo3(name = "setColor")]
fn set_color(args: &PyTuple) -> Result<(), PyErr> {
    Context::with_current(|m| {
        if let Result::<(u8, u8, u8), _>::Ok((r, g, b)) = args.extract() {
            m.set_color(Color::new(r, g, b))?;
        } else if let Result::<(&PyByteArray,), _>::Ok((bytearray,)) = args.extract() {
            if bytearray.len() == 3 * m.get_led_count() {
                // Safety: we are not modifying bytearray while accessing it
                unsafe {
                    m.set_led_colors(
                        bytearray
                            .as_bytes()
                            .chunks_exact(3)
                            .map(|rgb| Color::new(rgb[0], rgb[1], rgb[2]))
                            .collect(),
                    )?;
                }
            } else {
                return Err(RuntimeMethodError::InvalidByteArray.into());
            }
        } else {
            return Err(RuntimeMethodError::InvalidArguments { name: "setColor" }.into());
        }

        Ok(())
    })
}

/// Set a new image to process and determine new led colors
#[pyfunction]
#[pyo3(name = "setImage")]
fn set_image(width: u16, height: u16, data: &PyByteArray) -> Result<(), PyErr> {
    Context::with_current(|m| {
        // unwrap: we did all the necessary checks already
        m.set_image(
            RawImage::try_from((data.to_vec(), width as u32, height as u32))
                .map_err(|err| RuntimeMethodError::InvalidImageData(err))?,
        )?;

        Ok(())
    })
}

#[pymodule]
fn hyperion(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(abort, m)?)?;
    m.add_function(wrap_pyfunction!(set_color, m)?)?;
    m.add_function(wrap_pyfunction!(set_image, m)?)?;

    m.add("ledCount", Context::with_current(|m| m.get_led_count()))?;

    Ok(())
}

extern "C" fn hyperion_init() -> *mut pyo3::ffi::PyObject {
    unsafe { PyInit_hyperion() }
}

fn do_run<T>(
    methods: impl RuntimeMethods + 'static,
    args: serde_json::Value,
    f: impl FnOnce(Python) -> Result<T, PyErr>,
) -> Result<T, PyErr> {
    Context::with(methods, |ctx| {
        // Run the given code
        Python::with_gil(|py| {
            ctx.run(py, || {
                // Register arguments
                let hyperion_mod = py.import("hyperion")?;
                hyperion_mod.add("args", pythonize(py, &args)?)?;

                f(py)
            })
        })
    })
}

#[cfg(test)]
mod tests;
