use std::{cell::RefCell, panic, ptr::null_mut, sync::Once};

use drop_bomb::DropBomb;
use pyo3::prelude::*;

use super::{hyperion_init, RuntimeMethods};

static INITIALIZED_PYTHON: Once = Once::new();

thread_local! {
    /// Current effect context
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

/// Python effect module context
pub struct Context {
    tstate: *mut pyo3::ffi::PyThreadState,
    methods: Box<dyn RuntimeMethods>,
    bomb: DropBomb,
}

impl Context {
    unsafe fn new(_py: Python, methods: impl RuntimeMethods + 'static) -> Result<Self, ()> {
        // Get the main_state ptr
        let main_state = pyo3::ffi::PyEval_SaveThread();

        // Acquire GIL again
        pyo3::ffi::PyEval_RestoreThread(main_state);

        // Create new subinterp
        let tstate = pyo3::ffi::Py_NewInterpreter();

        // Restore GIL
        pyo3::ffi::PyThreadState_Swap(main_state);

        // Return object
        if tstate == null_mut() {
            Err(())
        } else {
            Ok(Self {
                tstate,
                methods: Box::new(methods),
                bomb: DropBomb::new("Context::release must be called before dropping it"),
            })
        }
    }

    unsafe fn release(&mut self, _py: Python) {
        // TODO: Stop sub threads?

        // Make this context subinterp current
        let main_thread = pyo3::ffi::PyThreadState_Swap(self.tstate);

        // Terminate it
        pyo3::ffi::Py_EndInterpreter(self.tstate);

        // Restore the main thread
        pyo3::ffi::PyThreadState_Swap(main_thread);

        // We're clear for dropping
        self.bomb.defuse();
    }

    pub fn run<U>(&self, _py: Python, f: impl FnOnce() -> U) -> U {
        unsafe {
            // Switch to the context thread
            let main_state = pyo3::ffi::PyThreadState_Swap(self.tstate);

            // Run user function
            let result = panic::catch_unwind(panic::AssertUnwindSafe(f));

            // Switch back to the main thread
            pyo3::ffi::PyThreadState_Swap(main_state);

            // Return result
            match result {
                Ok(result) => result,
                Err(panic) => panic::panic_any(panic),
            }
        }
    }

    pub fn with<U>(methods: impl RuntimeMethods + 'static, f: impl FnOnce(&Self) -> U) -> U {
        unsafe {
            // Initialize the Python interpreter global state
            INITIALIZED_PYTHON.call_once(|| {
                // Register our module through inittab
                pyo3::ffi::PyImport_AppendInittab(
                    b"hyperion\0".as_ptr() as *const _,
                    Some(hyperion_init),
                );

                pyo3::prepare_freethreaded_python();
            });

            let result = CONTEXT.with(|ctx| {
                // Initialize the thread-local state, i.e. interpreter
                *ctx.borrow_mut() = Some(Python::with_gil(|py| {
                    Self::new(py, methods).expect("failed initializing python subinterp")
                }));

                // Run user callback
                let result = {
                    let borrow = ctx.borrow();
                    let ctx = borrow.as_ref().unwrap();
                    panic::catch_unwind(panic::AssertUnwindSafe(|| f(ctx)))
                };

                // Free the interpreter
                if let Some(mut ctx) = ctx.borrow_mut().take() {
                    Python::with_gil(|py| {
                        ctx.release(py);
                    })
                }

                result
            });

            // Return result
            match result {
                Ok(result) => result,
                Err(panic) => panic::panic_any(panic),
            }
        }
    }

    pub fn with_current<U>(f: impl FnOnce(&dyn RuntimeMethods) -> U) -> U {
        CONTEXT.with(|ctx| f(&*ctx.borrow().as_ref().expect("no current context").methods))
    }
}
