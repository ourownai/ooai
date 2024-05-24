use pyo3::{prelude::*, types::PyTuple, PyErr, PyErrArguments, PyResult, Python};
use thiserror::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{oneshot, Mutex, RwLock, Semaphore};
use tokio::time::{timeout, Duration};
use num_cpus;
use pyo3::exceptions::PyValueError;

// Custom error type for the extension
#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("Failed to create Tokio runtime: {0}")]
    TokioRuntimeCreationError(String),
    #[error("Tokio block_on error: {0}")]
    TokioBlockOnError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Cancellation error: {0}")]
    CancellationError(String),
}

impl PyErrArguments for ExtensionError {
    fn arguments(&self, py: Python) -> &PyTuple {
        PyTuple::new(py, &[self.to_string()])
    }
}

impl std::convert::From<ExtensionError> for PyErr {
    fn from(err: ExtensionError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

// Define a Python class `RustTokioRuntime` using the `#[pyclass]` macro.
#[pyclass]
struct RustTokioRuntime {
    rt: Runtime,
    num_threads: usize,
    thread_name_prefix: String,
}

// Implement methods for the `RustTokioRuntime` class using the `#[pymethods]` macro.
#[pymethods]
impl RustTokioRuntime {
    // Define a constructor for the `RustTokioRuntime` class using the `#[new]` macro.
    #[new]
    fn new(num_threads: Option<usize>, thread_name_prefix: Option<String>) -> PyResult<Self> {
        let num_threads = num_threads.unwrap_or_else(num_cpus::get);
        let thread_name_prefix = thread_name_prefix.unwrap_or_else(|| "tokio-runtime-".to_string());
        Builder::new_multi_thread()
            .worker_threads(num_threads)
            .thread_name(thread_name_prefix.clone())
            .enable_all()
            .build()
            .map_err(|err| ExtensionError::TokioRuntimeCreationError(err.to_string()))
            .map(|rt| Self {
                rt,
                num_threads,
                thread_name_prefix,
            })
    }

    // Define a method `block_on` that runs an async future on the Tokio runtime with timeout and cancellation support.
    fn block_on(
        &self,
        future: PyObject,
        timeout_ms: Option<u64>,
        cancel_token: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let (cancel_sender, cancel_receiver) = oneshot::channel();
        if let Some(cancel_token) = cancel_token {
            let _ = py.allow_threads(|| {
                self.rt.spawn(async move {
                    let _ = cancel_token.call_method0("wait", ()).await;
                    let _ = cancel_sender.send(());
                });
            });
        }

        let result = py.allow_threads(|| {
            let future = async move {
                let result = match timeout_ms {
                    Some(ms) => {
                        let duration = Duration::from_millis(ms);
                        match timeout(duration, future.call0(py)).await {
                            Ok(result) => result.map_err(|err| ExtensionError::TokioBlockOnError(err.to_string())),
                            Err(_) => Err(ExtensionError::TimeoutError(
                                "Timeout exceeded".to_string(),
                            )),
                        }
                    }
                    None => future.call0(py).await.map_err(|err| ExtensionError::TokioBlockOnError(err.to_string())),
                };

                tokio::select! {
                    result = result => result,
                    _ = cancel_receiver => Err(ExtensionError::CancellationError("Cancelled".to_string())),
                }
            };

            self.rt.block_on(future)
        });

        result.map_err(Into::into)
    }

    // Define a method `shutdown` to gracefully shutdown the Tokio runtime.
    fn shutdown(&mut self) -> PyResult<()> {
        self.rt.shutdown_background();
        Ok(())
    }

    // Define a method `get_runtime_metrics` to retrieve runtime metrics.
    fn get_runtime_metrics(&self) -> PyResult<(usize, usize)> {
        let num_workers = self.rt.metrics().num_workers();
        let num_idle_workers = self.rt.metrics().num_idle_workers();
        Ok((num_workers, num_idle_workers))
    }

    // Define a method `create_mutex` to create a new Mutex.
    fn create_mutex(&self, value: PyObject) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mutex = Mutex::new(value);
        Py::new(py, mutex).map_err(Into::into)
    }

    // Define a method `create_rwlock` to create a new RwLock.
    fn create_rwlock(&self, value: PyObject) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let rwlock = RwLock::new(value);
        Py::new(py, rwlock).map_err(Into::into)
    }

    // Define a method `create_semaphore` to create a new Semaphore.
    fn create_semaphore(&self, permits: usize) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let semaphore = Semaphore::new(permits);
        Py::new(py, semaphore).map_err(Into::into)
    }

    // Implement async context management for the `RustTokioRuntime` class.
    fn __aenter__(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __aexit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<()> {
        self.shutdown()
    }
}

// Define a Python module `my_module` using the `#[pymodule]` macro.
#[pymodule]
fn my_module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustTokioRuntime>()?;
    Ok(())
}
