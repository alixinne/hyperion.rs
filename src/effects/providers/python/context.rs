use std::{
    cell::RefCell,
    panic,
    sync::{Arc, Once, Weak},
};

use drop_bomb::DropBomb;
use futures::Future;
use pyo3::prelude::*;

use super::{hyperion, RuntimeMethods};

static INITIALIZED_PYTHON: Once = Once::new();

thread_local! {
    /// Current effect context
    static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

/// Python effect module context
pub struct Context {
    tstate: *mut pyo3::ffi::PyThreadState,
    methods: Weak<dyn RuntimeMethods>,
    bomb: DropBomb,
}

impl Context {
    unsafe fn new(_py: Python, methods: Weak<dyn RuntimeMethods>) -> Result<Self, ()> {
        // Get the main_state ptr
        let main_state = pyo3::ffi::PyEval_SaveThread();

        // Acquire GIL again
        pyo3::ffi::PyEval_RestoreThread(main_state);

        // Create new subinterp
        let tstate = pyo3::ffi::Py_NewInterpreter();

        // Restore GIL
        pyo3::ffi::PyThreadState_Swap(main_state);

        // Return object
        if tstate.is_null() {
            Err(())
        } else {
            Ok(Self {
                tstate,
                methods,
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

    pub fn with<U>(methods: Arc<dyn RuntimeMethods>, f: impl FnOnce(&Self) -> U) -> U {
        unsafe {
            // Initialize the Python interpreter global state
            INITIALIZED_PYTHON.call_once(|| {
                // Register our module through inittab
                pyo3::append_to_inittab!(hyperion);
                Python::initialize();
            });

            let result = CONTEXT.with(|ctx| {
                // Initialize the thread-local state, i.e. interpreter
                *ctx.borrow_mut() = Some(Python::attach(|py| {
                    Self::new(py, Arc::downgrade(&methods))
                        .expect("failed initializing python subinterp")
                }));

                // Run user callback
                let result = {
                    let borrow = ctx.borrow();
                    let ctx = borrow.as_ref().unwrap();
                    panic::catch_unwind(panic::AssertUnwindSafe(|| f(ctx)))
                };

                // Free the interpreter
                if let Some(mut ctx) = ctx.borrow_mut().take() {
                    Python::attach(|py| {
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

    pub fn with_current<F, U>(f: impl FnOnce(Arc<dyn RuntimeMethods>) -> F) -> U
    where
        F: Future<Output = U>,
    {
        CONTEXT.with(|ctx| {
            futures::executor::block_on(f(ctx
                .borrow()
                .as_ref()
                .expect("no current context")
                .methods
                .upgrade()
                .expect("no current methods")))
        })
    }
}
