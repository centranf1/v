#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Opaque handle for compiled CENTRA-NF program (IR context).
 * Maintained by Rust, passed as *mut c_void to C callers.
 */
typedef struct CnfProgramHandle CnfProgramHandle;

/**
 * Opaque handle for runtime execution context.
 */
typedef struct CnfRuntimeHandle CnfRuntimeHandle;

/**
 * C-compatible error information structure.
 * Returned by all FFI functions.
 */
typedef struct CnfError {
  /**
   * Error code (from CnfErrorCode enum)
   */
  int32_t code;
  /**
   * Error message (allocated by Rust, must be freed by caller)
   */
  char *message;
} CnfError;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Compile CENTRA-NF source code to intermediate representation.
 *
 * # Arguments
 * * `source` - Null-terminated C string containing CENTRA-NF program
 * * `out_handle` - Output pointer to received compiled program handle
 *
 * # Returns
 * CnfError with error code and optional message.
 * If Ok, caller must eventually free handle with `cnf_free_program`.
 *
 * # Safety
 * - `source` must be valid null-terminated C string
 * - `out_handle` must be non-null and writable
 * - Caller responsible for freeing returned handle
 *
 * # Example (C code)
 * ```c
 * const char *source = "IDENTIFICATION DIVISION...";
 * void *handle = NULL;
 * CnfError err = cnf_compile(source, &handle);
 * if (err.code != 0) {
 *     printf("Compile error: %s\n", err.message);
 *     cnf_free_error(&err);
 *     return;
 * }
 * // Use handle...
 * cnf_free_program(handle);
 * ```
 */
struct CnfError cnf_compile(const char *source, struct CnfProgramHandle **out_handle);

/**
 * Free compiled program handle (deallocate IR context).
 *
 * # Safety
 * - `handle` must be a valid pointer returned by `cnf_compile`
 * - After calling this, `handle` must not be used again
 *
 * # Panics
 * - If `handle` is null (will return without error)
 */
void cnf_free_program(struct CnfProgramHandle *handle);

/**
 * Create new runtime execution context.
 *
 * # Returns
 * Pointer to new runtime handle. Must be freed with `cnf_free_runtime`.
 *
 * # Example (C code)
 * ```c
 * void *runtime = cnf_create_runtime();
 * // ... execute programs ...
 * cnf_free_runtime(runtime);
 * ```
 */
struct CnfRuntimeHandle *cnf_create_runtime(void);

/**
 * Free runtime execution context.
 *
 * # Safety
 * - `handle` must be a valid pointer returned by `cnf_create_runtime`
 * - After calling this, `handle` must not be used again
 */
void cnf_free_runtime(struct CnfRuntimeHandle *handle);

/**
 * Execute compiled program in runtime context.
 *
 * # Arguments
 * * `runtime_handle` - Runtime context (from `cnf_create_runtime`)
 * * `program_handle` - Compiled program (from `cnf_compile`)
 *
 * # Returns
 * CnfError with result or error code.
 *
 * # Safety
 * - Both handles must be valid and not yet freed
 * - `runtime_handle` and `program_handle` must come from corresponding create functions
 *
 * # Example (C code)
 * ```c
 * void *runtime = cnf_create_runtime();
 * void *program = NULL;
 * CnfError err = cnf_compile("...", &program);
 * if (err.code == 0) {
 *     err = cnf_execute(runtime, program);
 * }
 * ```
 */
struct CnfError cnf_execute(struct CnfRuntimeHandle *runtime_handle,
                            const struct CnfProgramHandle *program_handle);

/**
 * Compute SHA-256 hash of data.
 *
 * # Arguments
 * * `data` - Input buffer (arbitrary bytes)
 * * `data_len` - Length of input buffer
 * * `out_hash` - Output buffer (must be ≥ 32 bytes for hex representation = 64 bytes)
 * * `out_hash_capacity` - Size of output buffer
 *
 * # Returns
 * CnfError. If Ok, `out_hash` contains 64-byte hex string (null-terminated).
 *
 * # Safety
 * - `data` must point to valid buffer of at least `data_len` bytes
 * - `out_hash` must point to writable buffer of at least 65 bytes (64 hex + null terminator)
 * - This function does not allocate memory for output
 *
 * # Example (C code)
 * ```c
 * unsigned char data[] = {0x48, 0x65, 0x6c, 0x6c, 0x6f};  // "Hello"
 * char hash[65];  // 64 hex + null terminator
 * CnfError err = cnf_sha256(data, sizeof(data), hash, sizeof(hash));
 * if (err.code == 0) {
 *     printf("SHA-256: %s\n", hash);
 * }
 * ```
 */
struct CnfError cnf_sha256(const uint8_t *data,
                           uintptr_t data_len,
                           char *out_hash,
                           uintptr_t out_hash_capacity);

/**
 * Encrypt data using AES-256-GCM.
 *
 * # Arguments
 * * `plaintext` - Data to encrypt
 * * `plaintext_len` - Length of plaintext
 * * `out_ciphertext` - Output buffer for encrypted data (+ 12-byte nonce + 16-byte tag)
 * * `out_ciphertext_capacity` - Size of output buffer
 * * `out_ciphertext_len` - Output: actual size written
 *
 * # Returns
 * CnfError. If Ok, `out_ciphertext` contains [nonce(12) | ciphertext | tag(16)].
 *
 * # Safety
 * - All input pointers must be valid
 * - `out_ciphertext` capacity must be ≥ plaintext_len + 28
 * - Caller must provide output length pointer
 *
 * # Note
 * - Nonce is randomly generated per call (12 bytes)
 * - Authentication tag is appended (16 bytes)
 * - Total output = plaintext_len + 28
 */
struct CnfError cnf_aes256_encrypt(const uint8_t *plaintext,
                                   uintptr_t plaintext_len,
                                   uint8_t *out_ciphertext,
                                   uintptr_t out_ciphertext_capacity,
                                   uintptr_t *out_ciphertext_len);

/**
 * Decrypt data using AES-256-GCM.
 *
 * # Arguments
 * * `ciphertext` - Encrypted data (format: [nonce(12) | encrypted | tag(16)])
 * * `ciphertext_len` - Length of ciphertext
 * * `out_plaintext` - Output buffer for decrypted data
 * * `out_plaintext_capacity` - Size of output buffer
 * * `out_plaintext_len` - Output: actual size written
 *
 * # Returns
 * CnfError. If Ok, `out_plaintext` contains decrypted data.
 *
 * # Safety
 * - All input pointers must be valid
 * - `ciphertext_len` must be ≥ 28 (min: 12-byte nonce + 16-byte tag)
 * - `out_plaintext_capacity` must be ≥ ciphertext_len - 28
 *
 * # Errors
 * - InvalidArgument: Invalid input sizes
 * - CryptoError: Decryption failed or authentication failed
 */
struct CnfError cnf_aes256_decrypt(const uint8_t *ciphertext,
                                   uintptr_t ciphertext_len,
                                   uint8_t *out_plaintext,
                                   uintptr_t out_plaintext_capacity,
                                   uintptr_t *out_plaintext_len);

/**
 * Free error message string.
 *
 * # Safety
 * - `err` pointer must be from a CnfError returned by FFI function
 * - Must only be called once per error
 *
 * # Example (C code)
 * ```c
 * CnfError err = cnf_compile(...);
 * if (err.code != 0) {
 *     printf("Error: %s\n", err.message);
 * }
 * cnf_free_error(&err);
 * ```
 */
void cnf_free_error(struct CnfError *err);

/**
 * Get CENTRA-NF library version string.
 *
 * # Returns
 * Pointer to static string "CENTRA-NF 1.0.0"
 *
 * # Example (C code)
 * ```c
 * const char *version = cnf_version();
 * printf("Version: %s\n", version);
 * ```
 */
const char *cnf_version(void);

/**
 * Initialize CENTRA-NF library (for future use).
 *
 * # Returns
 * CnfError (always Ok for now)
 *
 * # Example (C code)
 * ```c
 * CnfError err = cnf_init();
 * if (err.code != 0) {
 *     printf("Init failed\n");
 * }
 * ```
 */
struct CnfError cnf_init(void);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
