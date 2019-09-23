//! Definition of the Effect type

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::{exceptions, PyErr};

use super::*;

/// Python Effect instance
#[pyclass]
pub struct Effect {
    /// Listener for Python method calls
    listener: Box<dyn EffectListener>,
    /// Number of LEDs managed by this instance
    led_count: usize,
    /// Instant at which the effect times out
    deadline: Option<Instant>,
    /// Flag to abort the running effect
    abort_requested: Arc<AtomicBool>,
    /// Arguments to the effect script
    args: PyObject,
}

impl Effect {
    /// Run the given effect code in a Python interpreter on the current thread
    ///
    /// # Parameters
    ///
    /// * `code`: source code for the effect
    /// * `listener`: method call listener for LED updates
    /// * `led_count`: number of LEDs managed by this effect
    /// * `args`: optional arguments to the effect
    /// * `deadline`: instant at which the effect times out
    /// * `abort_requested`: flag to abort the running effect
    pub fn run(
        code: &str,
        listener: Box<dyn EffectListener>,
        led_count: usize,
        args: Option<serde_json::Value>,
        deadline: Option<Instant>,
        abort_requested: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let hyperion = Py::new(
            py,
            Self {
                listener,
                led_count,
                deadline,
                abort_requested,
                args: args.map(|args| to_object(args, py)).to_object(py),
            },
        )
        .unwrap();

        let locals = [("hyperion", hyperion.to_object(py))].into_py_dict(py);

        py.run(code, None, Some(&locals)).map_err(|err| {
            if cfg!(test) {
                err.print(py);
                "Effect error".to_owned()
            } else {
                format!("{:?}", err)
            }
        })
    }

    /// Request that the effect be aborted
    pub fn request_abort(&mut self) {
        self.abort_requested.store(true, Ordering::SeqCst);
    }

    /// Checks the effect timeout, and requests aborting if necessary
    fn check_deadline(&mut self) {
        if self.deadline.map(|d| Instant::now() > d).unwrap_or(false) {
            trace!("terminating effect because deadline expired");
            self.request_abort();
        }
    }
}

#[pymethods]
#[allow(non_snake_case)]
impl Effect {
    /// Get the effect arguments
    #[getter]
    fn get_args(&self) -> &PyObject {
        &self.args
    }

    /// Get the LED count
    #[getter]
    fn get_ledCount(&self) -> usize {
        self.led_count
    }

    /// Set LED colors from RGB data
    #[args(args = "*")]
    fn setColor(&mut self, py: Python, args: &PyTuple) -> PyResult<()> {
        self.check_deadline();

        if args.len() == 3 {
            let r: u8 = args.get_item(0).to_object(py).extract(py)?;
            let g: u8 = args.get_item(1).to_object(py).extract(py)?;
            let b: u8 = args.get_item(2).to_object(py).extract(py)?;

            trace!("effect: set_rgb({}, {}, {})", r, g, b);
            self.listener.set_rgb((r, g, b))
        } else if args.len() == 1 {
            let obj = args.get_item(0).to_object(py);
            let led_data: &PyByteArray = obj.extract(py)?;

            // Check data length
            if led_data.len() != 3 * self.led_count {
                return Err(PyErr::new::<exceptions::RuntimeError, _>(
                    "Length of bytearray argument should be 3*ledCount",
                ));
            }

            trace!("effect: set_leds_rgb([{} bytes])", led_data.len());
            self.listener.set_leds_rgb(unsafe {
                &*(&led_data.to_vec()[..] as *const [u8] as *const [ByteRgb])
            })
        } else {
            Err(PyErr::new::<exceptions::RuntimeError, _>(
                "Could not parse arguments as color",
            ))
        }
    }

    /// Set LED colors from image data
    fn setImage(&mut self, width: u32, height: u32, data: &PyByteArray) -> PyResult<()> {
        self.check_deadline();

        if data.len() == 3 * width as usize * height as usize {
            let image = RawImage::try_from((data.to_vec(), width, height))
                .map_err(|err| PyErr::new::<exceptions::RuntimeError, _>(err.to_string()))?;

            self.listener.set_image(image)
        } else {
            Err(PyErr::new::<exceptions::RuntimeError, _>(
                "Length of bytearray argument should be 3*width*height",
            ))
        }
    }

    /// Check if the effect should abort or not
    fn abort(&mut self) -> PyResult<bool> {
        self.check_deadline();

        Ok(self.abort_requested.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    #[derive(Default)]
    struct DebugData {
        current: (u8, u8, u8),
    }

    #[derive(Clone)]
    struct DebugListener {
        data: Rc<RefCell<DebugData>>,
    }

    impl DebugListener {
        fn new() -> Self {
            Self {
                data: Rc::new(RefCell::new(DebugData::default())),
            }
        }
    }

    impl EffectListener for DebugListener {
        fn set_rgb(&mut self, rgb: (u8, u8, u8)) -> PyResult<()> {
            self.data.borrow_mut().current = rgb;
            Ok(())
        }

        fn set_leds_rgb(&mut self, leds: &[ByteRgb]) -> PyResult<()> {
            let rgb = &leds[0];
            self.data.borrow_mut().current = (rgb.r, rgb.g, rgb.b);
            Ok(())
        }

        fn set_image(&mut self, image: RawImage) -> PyResult<()> {
            self.data.borrow_mut().current = image.get_pixel(0, 0);
            Ok(())
        }
    }

    #[test]
    fn test_set_color_rgb() {
        let listener = Box::new(DebugListener::new());
        let abort_requested = Arc::new(AtomicBool::new(false));
        Effect::run(
            "hyperion.setColor(10, 20, 30)",
            listener.clone(),
            1,
            None,
            None,
            abort_requested,
        )
        .expect("running setColor failed");

        let data = listener.data.borrow();
        assert_eq!(data.current.0, 10);
        assert_eq!(data.current.1, 20);
        assert_eq!(data.current.2, 30);
    }

    #[test]
    fn test_set_color_leds() {
        let listener = Box::new(DebugListener::new());
        let abort_requested = Arc::new(AtomicBool::new(false));
        Effect::run(
            "hyperion.setColor(bytearray([10, 20, 30]))",
            listener.clone(),
            1,
            None,
            None,
            abort_requested,
        )
        .expect("running setColor failed");

        let data = listener.data.borrow();
        assert_eq!(data.current.0, 10);
        assert_eq!(data.current.1, 20);
        assert_eq!(data.current.2, 30);
    }

    #[test]
    fn test_set_image() {
        let listener = Box::new(DebugListener::new());
        let abort_requested = Arc::new(AtomicBool::new(false));
        Effect::run(
            "hyperion.setImage(1, 1, bytearray([10, 20, 30]))",
            listener.clone(),
            1,
            None,
            None,
            abort_requested,
        )
        .expect("running setImage failed");

        let data = listener.data.borrow();
        assert_eq!(data.current.0, 10);
        assert_eq!(data.current.1, 20);
        assert_eq!(data.current.2, 30);
    }

    #[test]
    fn test_abort() {
        let listener = Box::new(DebugListener::new());
        let abort_requested = Arc::new(AtomicBool::new(false));
        Effect::run(
            "assert hyperion.abort()",
            listener.clone(),
            1,
            None,
            Some(Instant::now() - Duration::from_millis(100)),
            abort_requested.clone(),
        )
        .expect("running abort failed");

        assert!(abort_requested.load(Ordering::SeqCst));
    }

    #[test]
    fn test_args() {
        let listener = Box::new(DebugListener::new());
        let abort_requested = Arc::new(AtomicBool::new(false));
        Effect::run(
            "assert hyperion.args['foo'] == 2",
            listener.clone(),
            1,
            Some(json!({ "foo": 2 })),
            None,
            abort_requested,
        )
        .expect("testing args failed");
    }
}
