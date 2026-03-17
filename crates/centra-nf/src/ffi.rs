//! # FFI Bridge for CENTRA-NF Universal SDK
//!
//! Provides C-compatible foreign function interface for multi-language bindings.
//! Enables Python, C++, C, and other languages to access CENTRA-NF functionality.
//!
//! ## Safety Guarantees
//! - All FFI functions are panic-proof (Result<T, E> → error codes)
//! - Memory management follows Rust allocation semantics
//! - No global mutable state
//! - Thread-safe (all functions are stateless or use Sync types)
//!
//! ## Design Pattern
//! - C callers receive: opaque handles (pointers) + error codes
//! - Rust maintains ownership internally
//! - Error information serialized to C-compatible format (error_code + message)
//!
//! # Safety
//! All FFI functions use #[no_mangle] and extern "C" for C calling convention.
//! Callers must:
//! 1. Check error codes before using returned pointers
//! 2. Free allocated memory via provided free functions
//! 3. Never use pointers after freeing

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::slice;

// ============================================================================
// TYPE DEFINITIONS: C-Compatible Representations
// ============================================================================

/// Opaque handle for compiled CENTRA-NF program (IR context).
/// Maintained by Rust, passed as *mut c_void to C callers.
pub struct CnfProgramHandle {
    instructions: Vec<cnf_compiler::Instruction>,
}

/// Opaque handle for runtime execution context.
pub struct CnfRuntimeHandle {
    runtime: cnf_runtime::Runtime,
}

/// Error code enumeration (C-compatible i32).
/// Must match unified error system from UNIFIED_ERROR_SYSTEM.md
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CnfErrorCode {
    /// Success (0)
    Ok = 0,
    /// Generic compilation error
    CompileError = 1,
    /// Invalid division order
    InvalidDivisionOrder = 2,
    /// Undefined variable
    UndefinedVariable = 3,
    /// Runtime execution error
    RuntimeError = 4,
    /// Cryptographic operation failed
    CryptoError = 5,
    /// Invalid memory access / bounds check
    MemoryError = 6,
    /// Invalid argument
    InvalidArgument = 7,
    /// Allocation failed
    AllocationFailed = 8,
    /// Invalid UTF-8 string
    InvalidUtf8 = 9,
    /// FFI error (invalid pointer, etc.)
    FfiError = 10,
}

/// C-compatible error information structure.
/// Returned by all FFI functions.
#[repr(C)]
pub struct CnfError {
    /// Error code (from CnfErrorCode enum)
    pub code: i32,
    /// Error message (allocated by Rust, must be freed by caller)
    pub message: *mut c_char,
}

impl CnfError {
    /// Create success response (no error).
    pub fn ok() -> Self {
        CnfError {
            code: CnfErrorCode::Ok as i32,
            message: ptr::null_mut(),
        }
    }

    /// Create error response with message.
    pub fn new(code: CnfErrorCode, msg: &str) -> Self {
        // Sanitize message by removing any null bytes
        let sanitized = msg.replace('\0', "");
        // Create CString from sanitized message (safe, no null bytes)
        let message = CString::new(sanitized)
            .unwrap_or_else(|_| CString::new("Error (message encoding failed)").expect("Static string has no nulls"));
        CnfError {
            code: code as i32,
            message: message.into_raw(),
        }
    }
}

// ============================================================================
// COMPILER FFI: Source Code → Intermediate Representation
// ============================================================================

/// Compile CENTRA-NF source code to intermediate representation.
///
/// # Arguments
/// * `source` - Null-terminated C string containing CENTRA-NF program
/// * `out_handle` - Output pointer to received compiled program handle
///
/// # Returns
/// CnfError with error code and optional message.
/// If Ok, caller must eventually free handle with `cnf_free_program`.
///
/// # Safety
/// - `source` must be valid null-terminated C string
/// - `out_handle` must be non-null and writable
/// - Caller responsible for freeing returned handle
///
/// # Example (C code)
/// ```c
/// const char *source = "IDENTIFICATION DIVISION...";
/// void *handle = NULL;
/// CnfError err = cnf_compile(source, &handle);
/// if (err.code != 0) {
///     printf("Compile error: %s\n", err.message);
///     cnf_free_error(&err);
///     return;
/// }
/// // Use handle...
/// cnf_free_program(handle);
/// ```
#[no_mangle]
pub extern "C" fn cnf_compile(
    source: *const c_char,
    out_handle: *mut *mut CnfProgramHandle,
) -> CnfError {
    // Validate pointers
    if source.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "source pointer is null",
        );
    }
    if out_handle.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "out_handle pointer is null",
        );
    }

    // Convert C string to Rust
    // SAFETY: Caller guarantees `source` is a valid, null-terminated C string from C caller
    let source_str = match unsafe { CStr::from_ptr(source) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            return CnfError::new(
                CnfErrorCode::InvalidUtf8,
                "source string is not valid UTF-8",
            )
        }
    };

    // Compile
    match cnf_compiler::compile(source_str) {
        Ok(instructions) => {
            let handle = Box::new(CnfProgramHandle { instructions });
            // SAFETY: out_handle is guaranteed to be a valid non-null pointer by caller
            unsafe {
                *out_handle = Box::into_raw(handle);
            }
            CnfError::ok()
        }
        Err(e) => CnfError::new(CnfErrorCode::CompileError, &format!("{}", e)),
    }
}

/// Free compiled program handle (deallocate IR context).
///
/// # Safety
/// - `handle` must be a valid pointer returned by `cnf_compile`
/// - After calling this, `handle` must not be used again
///
/// # Panics
/// - If `handle` is null (will return without error)
#[no_mangle]
pub extern "C" fn cnf_free_program(handle: *mut CnfProgramHandle) {
    if !handle.is_null() {
        // SAFETY: handle was previously returned by cnf_compile (Box::into_raw),
        // and this is the only place it's freed, ensuring no double-free
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

// ============================================================================
// RUNTIME FFI: Execute Compiled Programs
// ============================================================================

/// Create new runtime execution context.
///
/// # Returns
/// Pointer to new runtime handle. Must be freed with `cnf_free_runtime`.
///
/// # Example (C code)
/// ```c
/// void *runtime = cnf_create_runtime();
/// // ... execute programs ...
/// cnf_free_runtime(runtime);
/// ```
#[no_mangle]
pub extern "C" fn cnf_create_runtime() -> *mut CnfRuntimeHandle {
    let runtime = cnf_runtime::Runtime::new();
    let handle = Box::new(CnfRuntimeHandle { runtime });
    Box::into_raw(handle)
}

/// Free runtime execution context.
///
/// # Safety
/// - `handle` must be a valid pointer returned by `cnf_create_runtime`
/// - After calling this, `handle` must not be used again
#[no_mangle]
pub extern "C" fn cnf_free_runtime(handle: *mut CnfRuntimeHandle) {
    if !handle.is_null() {
        // SAFETY: handle was previously returned by cnf_create_runtime (Box::into_raw),
        // and this is the only place it's freed, ensuring no double-free
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

/// Execute compiled program in runtime context.
///
/// # Arguments
/// * `runtime_handle` - Runtime context (from `cnf_create_runtime`)
/// * `program_handle` - Compiled program (from `cnf_compile`)
///
/// # Returns
/// CnfError with result or error code.
///
/// # Safety
/// - Both handles must be valid and not yet freed
/// - `runtime_handle` and `program_handle` must come from corresponding create functions
///
/// # Example (C code)
/// ```c
/// void *runtime = cnf_create_runtime();
/// void *program = NULL;
/// CnfError err = cnf_compile("...", &program);
/// if (err.code == 0) {
///     err = cnf_execute(runtime, program);
/// }
/// ```
#[no_mangle]
pub extern "C" fn cnf_execute(
    runtime_handle: *mut CnfRuntimeHandle,
    program_handle: *const CnfProgramHandle,
) -> CnfError {
    // Validate pointers
    if runtime_handle.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "runtime_handle is null",
        );
    }
    if program_handle.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "program_handle is null",
        );
    }

    // Execute
    // SAFETY: Both handles are validated non-null above, and dereferencing them is safe
    // because the caller guarantees they come from corresponding create functions
    unsafe {
        match (*runtime_handle).runtime.execute_instructions(&(*program_handle).instructions) {
            Ok(_) => CnfError::ok(),
            Err(e) => CnfError::new(
                CnfErrorCode::RuntimeError,
                &format!("execution failed: {}", e),
            ),
        }
    }
}

// ============================================================================
// CRYPTOGRAPHIC FFI: SHA-256 and AES-256-GCM
// ============================================================================

/// Compute SHA-256 hash of data.
///
/// # Arguments
/// * `data` - Input buffer (arbitrary bytes)
/// * `data_len` - Length of input buffer
/// * `out_hash` - Output buffer (must be ≥ 32 bytes for hex representation = 64 bytes)
/// * `out_hash_capacity` - Size of output buffer
///
/// # Returns
/// CnfError. If Ok, `out_hash` contains 64-byte hex string (null-terminated).
///
/// # Safety
/// - `data` must point to valid buffer of at least `data_len` bytes
/// - `out_hash` must point to writable buffer of at least 65 bytes (64 hex + null terminator)
/// - This function does not allocate memory for output
///
/// # Example (C code)
/// ```c
/// unsigned char data[] = {0x48, 0x65, 0x6c, 0x6c, 0x6f};  // "Hello"
/// char hash[65];  // 64 hex + null terminator
/// CnfError err = cnf_sha256(data, sizeof(data), hash, sizeof(hash));
/// if (err.code == 0) {
///     printf("SHA-256: %s\n", hash);
/// }
/// ```
#[no_mangle]
pub extern "C" fn cnf_sha256(
    data: *const u8,
    data_len: usize,
    out_hash: *mut c_char,
    out_hash_capacity: usize,
) -> CnfError {
    // Validate pointers
    if data.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "data pointer is null",
        );
    }
    if out_hash.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "out_hash pointer is null",
        );
    }

    // Minimum output buffer for 64-byte hex + null terminator
    const MIN_HASH_LEN: usize = 65;
    if out_hash_capacity < MIN_HASH_LEN {
        return CnfError::new(
            CnfErrorCode::InvalidArgument,
            &format!("out_hash_capacity must be ≥ 65, got {}", out_hash_capacity),
        );
    }

    // Compute SHA-256
    // SAFETY: Caller guarantees `data` points to valid buffer of at least `data_len` bytes
    let input_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let hash_hex = cnf_security::sha256_hex(input_slice);

    // Copy to output buffer
    let hash_cstr = match CString::new(hash_hex.clone()) {
        Ok(s) => s,
        Err(_) => {
            return CnfError::new(
                CnfErrorCode::InvalidUtf8,
                "hash contains null bytes",
            )
        }
    };

    let hash_bytes = hash_cstr.as_bytes_with_nul();
    if hash_bytes.len() > out_hash_capacity {
        return CnfError::new(
            CnfErrorCode::InvalidArgument,
            "output buffer too small for hash",
        );
    }

    unsafe {
        // SAFETY: hash_bytes.len() is guaranteed to fit in out_hash_capacity (checked above)
        // and out_hash is guaranteed to be valid writable pointer by caller
        ptr::copy_nonoverlapping(
            hash_bytes.as_ptr() as *const c_char,
            out_hash,
            hash_bytes.len(),
        );
    }

    CnfError::ok()
}

/// Encrypt data using AES-256-GCM.
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `plaintext_len` - Length of plaintext
/// * `out_ciphertext` - Output buffer for encrypted data (+ 12-byte nonce + 16-byte tag)
/// * `out_ciphertext_capacity` - Size of output buffer
/// * `out_ciphertext_len` - Output: actual size written
///
/// # Returns
/// CnfError. If Ok, `out_ciphertext` contains [nonce(12) | ciphertext | tag(16)].
///
/// # Safety
/// - All input pointers must be valid
/// - `out_ciphertext` capacity must be ≥ plaintext_len + 28
/// - Caller must provide output length pointer
///
/// # Note
/// - Nonce is randomly generated per call (12 bytes)
/// - Authentication tag is appended (16 bytes)
/// - Total output = plaintext_len + 28
#[no_mangle]
pub extern "C" fn cnf_aes256_encrypt(
    plaintext: *const u8,
    plaintext_len: usize,
    out_ciphertext: *mut u8,
    out_ciphertext_capacity: usize,
    out_ciphertext_len: *mut usize,
) -> CnfError {
    // Validate pointers
    if plaintext.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "plaintext pointer is null",
        );
    }
    if out_ciphertext.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "out_ciphertext pointer is null",
        );
    }
    if out_ciphertext_len.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "out_ciphertext_len pointer is null",
        );
    }

    let required_capacity = plaintext_len.saturating_add(28);
    if out_ciphertext_capacity < required_capacity {
        return CnfError::new(
            CnfErrorCode::InvalidArgument,
            &format!(
                "out_ciphertext_capacity must be ≥ {}, got {}",
                required_capacity, out_ciphertext_capacity
            ),
        );
    }

    // Perform encryption
    // SAFETY: Caller guarantees `plaintext` points to valid buffer of at least `plaintext_len` bytes
    let input = unsafe { slice::from_raw_parts(plaintext, plaintext_len) };
    match cnf_security::encrypt_aes256(input) {
        Ok(encrypted) => {
            if encrypted.len() > out_ciphertext_capacity {
                return CnfError::new(
                    CnfErrorCode::InvalidArgument,
                    "encrypted output exceeds buffer capacity",
                );
            }
            unsafe {
                // SAFETY: encrypted.len() is validated to fit in out_ciphertext_capacity (checked above)
                // and out_ciphertext is guaranteed to be valid writable pointer by caller
                ptr::copy_nonoverlapping(
                    encrypted.as_ptr(),
                    out_ciphertext,
                    encrypted.len(),
                );
                *out_ciphertext_len = encrypted.len();
            }
            CnfError::ok()
        }
        Err(_) => CnfError::new(
            CnfErrorCode::CryptoError,
            "AES-256-GCM encryption failed",
        ),
    }
}

/// Decrypt data using AES-256-GCM.
///
/// # Arguments
/// * `ciphertext` - Encrypted data (format: [nonce(12) | encrypted | tag(16)])
/// * `ciphertext_len` - Length of ciphertext
/// * `out_plaintext` - Output buffer for decrypted data
/// * `out_plaintext_capacity` - Size of output buffer
/// * `out_plaintext_len` - Output: actual size written
///
/// # Returns
/// CnfError. If Ok, `out_plaintext` contains decrypted data.
///
/// # Safety
/// - All input pointers must be valid
/// - `ciphertext_len` must be ≥ 28 (min: 12-byte nonce + 16-byte tag)
/// - `out_plaintext_capacity` must be ≥ ciphertext_len - 28
///
/// # Errors
/// - InvalidArgument: Invalid input sizes
/// - CryptoError: Decryption failed or authentication failed
#[no_mangle]
pub extern "C" fn cnf_aes256_decrypt(
    ciphertext: *const u8,
    ciphertext_len: usize,
    out_plaintext: *mut u8,
    out_plaintext_capacity: usize,
    out_plaintext_len: *mut usize,
) -> CnfError {
    // Validate pointers
    if ciphertext.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "ciphertext pointer is null",
        );
    }
    if out_plaintext.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "out_plaintext pointer is null",
        );
    }
    if out_plaintext_len.is_null() {
        return CnfError::new(
            CnfErrorCode::FfiError,
            "out_plaintext_len pointer is null",
        );
    }

    // Validate sizes (minimum: 12-byte nonce + 16-byte tag)
    if ciphertext_len < 28 {
        return CnfError::new(
            CnfErrorCode::InvalidArgument,
            "ciphertext must be at least 28 bytes (nonce + tag)",
        );
    }

    let plaintext_size = ciphertext_len - 28;
    if out_plaintext_capacity < plaintext_size {
        return CnfError::new(
            CnfErrorCode::InvalidArgument,
            &format!(
                "out_plaintext_capacity must be ≥ {}, got {}",
                plaintext_size, out_plaintext_capacity
            ),
        );
    }

    // Perform decryption
    // SAFETY: Caller guarantees `ciphertext` points to valid buffer of at least `ciphertext_len` bytes
    let input = unsafe { slice::from_raw_parts(ciphertext, ciphertext_len) };
    match cnf_security::decrypt_aes256(input) {
        Ok(plaintext) => {
            if plaintext.len() > out_plaintext_capacity {
                return CnfError::new(
                    CnfErrorCode::InvalidArgument,
                    "decrypted output exceeds buffer capacity",
                );
            }
            unsafe {
                // SAFETY: plaintext.len() is validated to fit in out_plaintext_capacity (checked above)
                // and out_plaintext is guaranteed to be valid writable pointer by caller
                ptr::copy_nonoverlapping(
                    plaintext.as_ptr(),
                    out_plaintext,
                    plaintext.len(),
                );
                *out_plaintext_len = plaintext.len();
            }
            CnfError::ok()
        }
        Err(_) => CnfError::new(
            CnfErrorCode::CryptoError,
            "AES-256-GCM decryption failed (invalid ciphertext or tag)",
        ),
    }
}

// ============================================================================
// ERROR MANAGEMENT FFI
// ============================================================================

/// Free error message string.
///
/// # Safety
/// - `err` pointer must be from a CnfError returned by FFI function
/// - Must only be called once per error
///
/// # Example (C code)
/// ```c
/// CnfError err = cnf_compile(...);
/// if (err.code != 0) {
///     printf("Error: %s\n", err.message);
/// }
/// cnf_free_error(&err);
/// ```
#[no_mangle]
pub extern "C" fn cnf_free_error(err: *mut CnfError) {
    if !err.is_null() {
        // SAFETY: err is guaranteed to be valid pointer from FFI return,
        // and message is either null or valid CString obtained from CString::into_raw
        unsafe {
            if !(*err).message.is_null() {
                let _ = CString::from_raw((*err).message);
            }
        }
    }
}

// ============================================================================
// LIBRARY INITIALIZATION & VERSION
// ============================================================================

/// Get CENTRA-NF library version string.
///
/// # Returns
/// Pointer to static string "CENTRA-NF 1.0.0"
///
/// # Example (C code)
/// ```c
/// const char *version = cnf_version();
/// printf("Version: %s\n", version);
/// ```
#[no_mangle]
pub extern "C" fn cnf_version() -> *const c_char {
    b"CENTRA-NF 1.0.0\0".as_ptr() as *const c_char
}

/// Initialize CENTRA-NF library (for future use).
///
/// # Returns
/// CnfError (always Ok for now)
///
/// # Example (C code)
/// ```c
/// CnfError err = cnf_init();
/// if (err.code != 0) {
///     printf("Init failed\n");
/// }
/// ```
#[no_mangle]
pub extern "C" fn cnf_init() -> CnfError {
    CnfError::ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_conversion() {
        assert_eq!(CnfErrorCode::Ok as i32, 0);
        assert_eq!(CnfErrorCode::CompileError as i32, 1);
    }

    #[test]
    fn test_cnf_version() {
        unsafe {
            let version = CStr::from_ptr(cnf_version()).to_str().unwrap();
            assert_eq!(version, "CENTRA-NF 1.0.0");
        }
    }
}
