// PyO3 Python Bindings for CENTRA-NF
//
// This module provides Python access to CENTRA-NF functionality via PyO3.
// Build with: cargo build --release --features python-bindings
// Results in: maturin develop
//
// Usage from Python:
//   import centra_nf
//   program = centra_nf.compile("IDENTIFICATION DIVISION...")
//   runtime = centra_nf.Runtime()
//   runtime.execute(program)

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::ffi::{CStr, CString};

mod ffi {
    use crate::*;

    /// Rust wrapper around C FFI (for internal use only)
    pub fn compile_rs(source: &str) -> Result<*mut crate::ffi::CnfProgramHandle, String> {
        let c_source = CString::new(source)
            .map_err(|e| format!("Invalid source string: {}", e))?;

        let mut out_handle: *mut crate::ffi::CnfProgramHandle = std::ptr::null_mut();
        let err = unsafe { crate::ffi::cnf_compile(c_source.as_ptr(), &mut out_handle) };

        if err.code != 0 {
            let msg = if !err.message.is_null() {
                unsafe { CStr::from_ptr(err.message).to_string_lossy().to_string() }
            } else {
                "Unknown FFI error".to_string()
            };
            unsafe { crate::ffi::cnf_free_error(&err); }
            Err(msg)
        } else {
            Ok(out_handle)
        }
    }
}

/// Python exception types matching CENTRA-NF error codes
#[derive(Clone)]
pub enum PyErrorCode {
    Ok,
    CompileError,
    InvalidDivisionOrder,
    UndefinedVariable,
    RuntimeError,
    CryptoError,
    MemoryError,
    InvalidArgument,
    AllocationFailed,
    InvalidUtf8,
    FfiError,
}

impl PyErrorCode {
    fn to_exception(&self) -> &'static str {
        match self {
            PyErrorCode::Ok => "Ok",
            PyErrorCode::CompileError => "CompileError",
            PyErrorCode::InvalidDivisionOrder => "InvalidDivisionOrder",
            PyErrorCode::UndefinedVariable => "UndefinedVariable",
            PyErrorCode::RuntimeError => "RuntimeError",
            PyErrorCode::CryptoError => "CryptoError",
            PyErrorCode::MemoryError => "MemoryError",
            PyErrorCode::InvalidArgument => "InvalidArgument",
            PyErrorCode::AllocationFailed => "AllocationFailed",
            PyErrorCode::InvalidUtf8 => "InvalidUtf8",
            PyErrorCode::FfiError => "FfiError",
        }
    }
}

/// Compiled CENTRA-NF program (opaque handle)
#[pyclass(name = "Program")]
pub struct PyProgram {
    #[pyo3(get)]
    source: String,
    handle: *mut crate::ffi::CnfProgramHandle,
}

impl Drop for PyProgram {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                crate::ffi::cnf_free_program(self.handle);
            }
        }
    }
}

#[pymethods]
impl PyProgram {
    /// Get the original source code
    fn get_source(&self) -> String {
        self.source.clone()
    }

    /// String representation
    fn __repr__(&self) -> String {
        format!("Program(len={})", self.source.len())
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Runtime execution context
#[pyclass(name = "Runtime")]
pub struct PyRuntime {
    handle: *mut crate::ffi::CnfRuntimeHandle,
}

impl Drop for PyRuntime {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                crate::ffi::cnf_free_runtime(self.handle);
            }
        }
    }
}

#[pymethods]
impl PyRuntime {
    /// Create new runtime context
    #[new]
    fn new() -> PyResult<Self> {
        let handle = unsafe { crate::ffi::cnf_create_runtime() };
        if handle.is_null() {
            return Err(PyRuntimeError::new_err("Failed to create runtime"));
        }
        Ok(PyRuntime { handle })
    }

    /// Execute compiled program
    fn execute(&self, program: &PyProgram) -> PyResult<()> {
        let err = unsafe { crate::ffi::cnf_execute(self.handle, program.handle) };

        if err.code != 0 {
            let msg = if !err.message.is_null() {
                unsafe { CStr::from_ptr(err.message).to_string_lossy().to_string() }
            } else {
                "Unknown runtime error".to_string()
            };
            unsafe { crate::ffi::cnf_free_error(&err); }
            Err(PyRuntimeError::new_err(msg))
        } else {
            Ok(())
        }
    }

    /// String representation
    fn __repr__(&self) -> String {
        "Runtime()".to_string()
    }
}

/// Compile CENTRA-NF source code
#[pyfunction]
fn compile(source: String) -> PyResult<Py<PyProgram>> {
    let handle = ffi::compile_rs(&source)
        .map_err(|e| PyRuntimeError::new_err(e))?;

    Python::with_gil(|py| {
        let program = PyProgram { source, handle };
        Py::new(py, program)
    })
}

/// Compute SHA-256 hash (returns hex string)
#[pyfunction]
fn sha256(data: Vec<u8>) -> PyResult<String> {
    let mut hash_buf = [0u8; 65];  // 64 hex + null terminator

    let err = unsafe {
        crate::ffi::cnf_sha256(
            data.as_ptr(),
            data.len(),
            hash_buf.as_mut_ptr() as *mut i8,
            hash_buf.len(),
        )
    };

    if err.code != 0 {
        let msg = if !err.message.is_null() {
            unsafe { CStr::from_ptr(err.message).to_string_lossy().to_string() }
        } else {
            "SHA-256 error".to_string()
        };
        unsafe { crate::ffi::cnf_free_error(&err); }
        return Err(PyRuntimeError::new_err(msg));
    }

    // Extract string up to null terminator
    let hash_str = unsafe {
        CStr::from_ptr(hash_buf.as_ptr() as *const i8)
            .to_string_lossy()
            .to_string()
    };
    Ok(hash_str)
}

/// Encrypt data using AES-256-GCM (returns bytes)
#[pyfunction]
fn aes256_encrypt(plaintext: Vec<u8>) -> PyResult<Vec<u8>> {
    let mut ciphertext = vec![0u8; plaintext.len() + 28];  // nonce + encrypted + tag
    let mut ciphertext_len = 0usize;

    let err = unsafe {
        crate::ffi::cnf_aes256_encrypt(
            plaintext.as_ptr(),
            plaintext.len(),
            ciphertext.as_mut_ptr(),
            ciphertext.capacity(),
            &mut ciphertext_len,
        )
    };

    if err.code != 0 {
        let msg = if !err.message.is_null() {
            unsafe { CStr::from_ptr(err.message).to_string_lossy().to_string() }
        } else {
            "AES-256 encryption error".to_string()
        };
        unsafe { crate::ffi::cnf_free_error(&err); }
        return Err(PyRuntimeError::new_err(msg));
    }

    ciphertext.truncate(ciphertext_len);
    Ok(ciphertext)
}

/// Decrypt data using AES-256-GCM (returns bytes)
#[pyfunction]
fn aes256_decrypt(ciphertext: Vec<u8>) -> PyResult<Vec<u8>> {
    if ciphertext.len() < 28 {
        return Err(PyValueError::new_err(
            "Ciphertext too short (minimum 28 bytes: 12-byte nonce + 16-byte tag)"
        ));
    }

    let mut plaintext = vec![0u8; ciphertext.len() - 28];
    let mut plaintext_len = 0usize;

    let err = unsafe {
        crate::ffi::cnf_aes256_decrypt(
            ciphertext.as_ptr(),
            ciphertext.len(),
            plaintext.as_mut_ptr(),
            plaintext.capacity(),
            &mut plaintext_len,
        )
    };

    if err.code != 0 {
        let msg = if !err.message.is_null() {
            unsafe { CStr::from_ptr(err.message).to_string_lossy().to_string() }
        } else {
            "AES-256 decryption error (possible data tampering)".to_string()
        };
        unsafe { crate::ffi::cnf_free_error(&err); }
        return Err(PyRuntimeError::new_err(msg));
    }

    plaintext.truncate(plaintext_len);
    Ok(plaintext)
}

/// Get version string
#[pyfunction]
fn version() -> String {
    let v = unsafe { crate::ffi::cnf_version() };
    unsafe { CStr::from_ptr(v).to_string_lossy().to_string() }
}

/// Initialize library
#[pyfunction]
fn init() -> PyResult<()> {
    let err = unsafe { crate::ffi::cnf_init() };
    if err.code != 0 {
        let msg = if !err.message.is_null() {
            unsafe { CStr::from_ptr(err.message).to_string_lossy().to_string() }
        } else {
            "Initialization error".to_string()
        };
        unsafe { crate::ffi::cnf_free_error(&err); }
        return Err(PyRuntimeError::new_err(msg));
    }
    Ok(())
}

/// CENTRA-NF Python Module
#[pymodule]
fn centra_nf(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(compile))?;
    m.add_wrapped(wrap_pyfunction!(sha256))?;
    m.add_wrapped(wrap_pyfunction!(aes256_encrypt))?;
    m.add_wrapped(wrap_pyfunction!(aes256_decrypt))?;
    m.add_wrapped(wrap_pyfunction!(version))?;
    m.add_wrapped(wrap_pyfunction!(init))?;
    m.add_class::<PyProgram>()?;
    m.add_class::<PyRuntime>()?;

    // Add version string
    m.add("__version__", "1.0.0")?;

    // Add module docstring
    m.add(
        "__doc__",
        r#"
CENTRA-NF Python Module

A high-performance compilation and cryptography framework for COBOL and data-intensive applications.

Quick Start:
    import centra_nf
    
    # Compile a program
    program = centra_nf.compile('''
    IDENTIFICATION DIVISION.
    ENVIRONMENT DIVISION.
    DATA DIVISION.
    PROCEDURE DIVISION.
    ''')
    
    # Execute it
    runtime = centra_nf.Runtime()
    runtime.execute(program)
    
    # Cryptography
    hash_hex = centra_nf.sha256(b"Hello, World!")
    encrypted = centra_nf.aes256_encrypt(b"Secret message")
    decrypted = centra_nf.aes256_decrypt(encrypted)
    
    # Info
    print(centra_nf.version())

Error Handling:
    All functions raise exceptions on error:
    - RuntimeError: Compilation, execution, or cryptography errors
    - ValueError: Invalid input parameters

Cryptography Details:
    - SHA-256: Deterministic, constant-time
    - AES-256-GCM: With authentication, prevents tampering
    - Format: [nonce(12) | ciphertext | tag(16)]
"#,
    )?;

    Ok(())
}
