/// Python Bindings for CENTRA-NF using PyO3
///
/// This module wraps the core FFI functions for Python 3.8+
/// Provides Pythonic interfaces while maintaining Military-Grade safety
///
/// # Building Python Module
/// ```bash
/// pip install maturin
/// maturin develop --release
/// ```
///
/// # Python Usage
/// ```python
/// import centra_nf
/// 
/// # Compile
/// program = centra_nf.compile("IDENTIFICATION DIVISION...")
/// 
/// # Hash
/// digest = centra_nf.sha256(b"data")
/// 
/// # Encrypt/Decrypt
/// encrypted = centra_nf.encrypt(b"secret")
/// decrypted = centra_nf.decrypt(encrypted)
/// ```

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::PyBytes;
use pyo3::types::PyModule;

// ============================================================================
// MODULE INITIALIZATION
// ============================================================================

#[pymodule]
fn core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", "1.0.0")?;
    m.add("__doc__", "CENTRA-NF Universal SDK - Cryptographic operations (v1.0.0)")?;

    // Register module-level functions
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(build_info, m)?)?;
    m.add_function(wrap_pyfunction!(sha256, m)?)?;
    m.add_function(wrap_pyfunction!(encrypt, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt, m)?)?;

    Ok(())
}

// ============================================================================
// ERRORS & EXCEPTIONS
// ============================================================================

/// Helper function to convert FFI error to Python RuntimeError
fn ffi_error(err: &crate::ffi::CnfError) -> PyErr {
    let msg = if !err.message.is_null() {
        unsafe { std::ffi::CStr::from_ptr(err.message) }
            .to_string_lossy()
            .to_string()
    } else {
        format!("FFI error code: {}", err.code)
    };
    PyRuntimeError::new_err(msg)
}

// ============================================================================
// MODULE-LEVEL FUNCTIONS
// ============================================================================

/// Get CENTRA-NF version
///
/// # Python
/// ```python
/// >>> import centra_nf
/// >>> centra_nf.version()
/// 'CENTRA-NF 1.0.0'
/// ```
#[pyfunction]
fn version() -> &'static str {
    "CENTRA-NF 1.0.0"
}

/// Get build information
///
/// # Python
/// ```python
/// >>> centra_nf.build_info()
/// 'CENTRA-NF 1.0.0\nBuilt: 1.0.0\nProfile: release (Military-Grade optimized)'
/// ```
#[pyfunction]
fn build_info() -> &'static str {
    concat!(
        "CENTRA-NF 1.0.0\n",
        "Built: ", env!("CARGO_PKG_VERSION"), "\n",
        "Profile: release (Military-Grade optimized)"
    )
}

/// Compute SHA-256 hash (constant-time)
///
/// # Arguments
/// - `data` (bytes): Data to hash
///
/// # Returns
/// - str: Lowercase hex string (64 characters)
///
/// # Example
/// ```python
/// >>> centra_nf.sha256(b"Hello, World!")
/// 'dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f'
/// ```
#[pyfunction]
fn sha256(data: &[u8]) -> PyResult<String> {
    let mut hash = vec![0u8; 65];

    let err = unsafe {
        crate::ffi::cnf_sha256(
            data.as_ptr(),
            data.len(),
            hash.as_mut_ptr() as *mut i8,
            hash.len(),
        )
    };

    if err.code != 0 {
        return Err(ffi_error(&err));
    }

    // Convert null-terminated C string to Rust String
    let hash_str = unsafe {
        std::ffi::CStr::from_ptr(hash.as_ptr() as *const i8)
            .to_string_lossy()
            .to_string()
    };

    Ok(hash_str)
}

/// Encrypt data with AES-256-GCM (random nonce per encryption)
///
/// # Arguments
/// - `plaintext` (bytes): Data to encrypt
///
/// # Returns
/// - bytes: Ciphertext (includes nonce + tag)
///
/// # Example
/// ```python
/// >>> encrypted = centra_nf.encrypt(b"Secret message")
/// >>> len(encrypted)  # plaintext_len + nonce(12) + tag(16)
/// 30
/// ```
#[pyfunction]
fn encrypt(py: Python, plaintext: &[u8]) -> PyResult<PyObject> {
    let max_len = plaintext.len() + 128; // Extra space for nonce + tag
    let mut ciphertext = vec![0u8; max_len];
    let mut ct_len: usize = 0;

    let err = unsafe {
        crate::ffi::cnf_aes256_encrypt(
            plaintext.as_ptr(),
            plaintext.len(),
            ciphertext.as_mut_ptr(),
            max_len,
            &mut ct_len,
        )
    };

    if err.code != 0 {
        return Err(ffi_error(&err));
    }

    ciphertext.truncate(ct_len);
    Ok(PyBytes::new(py, &ciphertext).into())
}

/// Decrypt data with AES-256-GCM
///
/// # Arguments
/// - `ciphertext` (bytes): Data from encrypt()
///
/// # Returns
/// - bytes: Original plaintext
///
/// # Raises
/// - RuntimeError: If decryption or authentication fails
///
/// # Example
/// ```python
/// >>> encrypted = centra_nf.encrypt(b"Secret message")
/// >>> decrypted = centra_nf.decrypt(encrypted)
/// >>> decrypted
/// b'Secret message'
/// ```
#[pyfunction]
fn decrypt(py: Python, ciphertext: &[u8]) -> PyResult<PyObject> {
    let max_len = ciphertext.len();
    let mut plaintext = vec![0u8; max_len];
    let mut pt_len: usize = 0;

    let err = unsafe {
        crate::ffi::cnf_aes256_decrypt(
            ciphertext.as_ptr(),
            ciphertext.len(),
            plaintext.as_mut_ptr(),
            max_len,
            &mut pt_len,
        )
    };

    if err.code != 0 {
        return Err(ffi_error(&err));
    }

    plaintext.truncate(pt_len);
    Ok(PyBytes::new(py, &plaintext).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "CENTRA-NF 1.0.0");
    }

    #[test]
    fn test_build_info_contains_version() {
        assert!(build_info().contains("CENTRA-NF"));
    }
}
