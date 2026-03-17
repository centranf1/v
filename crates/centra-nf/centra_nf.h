/*
 * CENTRA-NF FFI Header File
 * C/C++ Foreign Function Interface
 *
 * This header provides complete declarations for CENTRA-NF library.
 * Include this in your C/C++ projects to use CENTRA-NF functionality.
 *
 * Example (C):
 *     #include "centra_nf.h"
 *     #include <stdio.h>
 *
 *     int main() {
 *         const char *version = cnf_version();
 *         printf("CENTRA-NF %s\n", version);
 *
 *         CnfError err = cnf_init();
 *         if (err.code != 0) {
 *             printf("Init failed: %s\n", err.message);
 *             cnf_free_error(&err);
 *             return 1;
 *         }
 *         return 0;
 *     }
 *
 * Compile (Linux):
 *     gcc -o myapp myapp.c -lcentra_nf -L /path/to/libcentra_nf.so
 */

#ifndef CENTRA_NF_H
#define CENTRA_NF_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * ERROR CODES
 * ============================================================================ */

/**
 * Error code enumeration.
 * Success = 0, all errors > 0.
 */
typedef enum {
    CNF_OK = 0,                      /**< Success */
    CNF_COMPILE_ERROR = 1,           /**< Compilation error */
    CNF_INVALID_DIVISION_ORDER = 2,  /**< Invalid division order */
    CNF_UNDEFINED_VARIABLE = 3,      /**< Referenced variable not defined */
    CNF_RUNTIME_ERROR = 4,           /**< Runtime execution error */
    CNF_CRYPTO_ERROR = 5,            /**< Cryptographic operation failed */
    CNF_MEMORY_ERROR = 6,            /**< Memory access / bounds violation */
    CNF_INVALID_ARGUMENT = 7,        /**< Invalid function argument */
    CNF_ALLOCATION_FAILED = 8,       /**< Memory allocation failed */
    CNF_INVALID_UTF8 = 9,            /**< String is not valid UTF-8 */
    CNF_FFI_ERROR = 10,              /**< FFI error (null pointer, etc.) */
} cnf_error_code_e;

/**
 * Error information structure.
 * Returned by all FFI functions.
 *
 * Fields:
 *   - code: Error code (0 = success)
 *   - message: Error message string (NULL if no error)
 *
 * Important: If error.message is non-NULL, must call cnf_free_error() to free.
 */
typedef struct {
    int32_t code;       /**< Error code (from cnf_error_code_e) */
    char *message;      /**< Error message (must be freed with cnf_free_error) */
} cnf_error_t;

/* ============================================================================
 * OPAQUE HANDLES
 * ============================================================================ */

/**
 * Opaque handle for compiled CENTRA-NF program (IR context).
 * Returned by cnf_compile(). Must be freed with cnf_free_program().
 */
typedef void cnf_program_t;

/**
 * Opaque handle for runtime execution context.
 * Created by cnf_create_runtime(). Must be freed with cnf_free_runtime().
 */
typedef void cnf_runtime_t;

/* ============================================================================
 * COMPILER API
 * ============================================================================ */

/**
 * Compile CENTRA-NF source code to intermediate representation.
 *
 * Args:
 *   source       - Null-terminated C string containing CENTRA-NF program
 *   out_program  - Output: pointer to compiled program handle
 *
 * Returns:
 *   Error code and message (see cnf_error_t).
 *   If successful (code == 0), caller must free handle with cnf_free_program().
 *
 * Example:
 *   cnf_program_t *program = NULL;
 *   cnf_error_t err = cnf_compile("IDENTIFICATION DIVISION...", &program);
 *   if (err.code != 0) {
 *       fprintf(stderr, "Compile failed: %s\n", err.message);
 *       cnf_free_error(&err);
 *       return 1;
 *   }
 *   // Use program...
 *   cnf_free_program(program);
 */
cnf_error_t cnf_compile(const char *source, cnf_program_t **out_program);

/**
 * Free compiled program handle (deallocate IR context).
 *
 * Args:
 *   program - Handle from cnf_compile() (or NULL, which is safe)
 *
 * Note: After calling this, pointer must not be used again.
 */
void cnf_free_program(cnf_program_t *program);

/* ============================================================================
 * RUNTIME API
 * ============================================================================ */

/**
 * Create new runtime execution context.
 *
 * Returns:
 *   Pointer to new runtime handle.
 *   Must be freed with cnf_free_runtime().
 *
 * Note: Always succeeds (memory allocation failure will abort).
 *
 * Example:
 *   cnf_runtime_t *runtime = cnf_create_runtime();
 *   // ... execute programs ...
 *   cnf_free_runtime(runtime);
 */
cnf_runtime_t *cnf_create_runtime(void);

/**
 * Free runtime execution context.
 *
 * Args:
 *   runtime - Handle from cnf_create_runtime() (or NULL, which is safe)
 *
 * Note: After calling this, pointer must not be used again.
 */
void cnf_free_runtime(cnf_runtime_t *runtime);

/**
 * Execute compiled program in runtime context.
 *
 * Args:
 *   runtime  - Runtime context (from cnf_create_runtime)
 *   program  - Compiled program (from cnf_compile)
 *
 * Returns:
 *   Error code and message (see cnf_error_t).
 *   Success = code 0.
 *
 * Example:
 *   cnf_error_t err = cnf_execute(runtime, program);
 *   if (err.code != 0) {
 *       fprintf(stderr, "Execution failed: %s\n", err.message);
 *       cnf_free_error(&err);
 *   }
 */
cnf_error_t cnf_execute(cnf_runtime_t *runtime, const cnf_program_t *program);

/* ============================================================================
 * CRYPTOGRAPHIC API
 * ============================================================================ */

/**
 * Compute SHA-256 hash of data.
 *
 * Args:
 *   data             - Input buffer (arbitrary bytes)
 *   data_len         - Length of input buffer
 *   out_hash         - Output buffer (must be >= 65 bytes: 64 hex + null terminator)
 *   out_hash_capacity - Capacity of output buffer
 *
 * Returns:
 *   Error code and message.
 *   If successful, out_hash contains 64-character hex string (null-terminated).
 *
 * Requirements:
 *   - out_hash_capacity >= 65 (64 hex + null terminator)
 *   - output buffer is NOT dynamically allocated (caller provides buffer)
 *
 * Example:
 *   unsigned char data[] = "Hello, World!";
 *   char hash[65];  // 64 hex + null
 *   cnf_error_t err = cnf_sha256(data, sizeof(data)-1, hash, sizeof(hash));
 *   if (err.code == 0) {
 *       printf("SHA-256: %s\n", hash);
 *   }
 */
cnf_error_t cnf_sha256(
    const uint8_t *data,
    size_t data_len,
    char *out_hash,
    size_t out_hash_capacity
);

/**
 * Encrypt data using AES-256-GCM.
 *
 * Args:
 *   plaintext               - Data to encrypt
 *   plaintext_len           - Length of plaintext
 *   out_ciphertext          - Output buffer for encrypted data
 *   out_ciphertext_capacity - Size of output buffer
 *   out_ciphertext_len      - Output: actual size written
 *
 * Returns:
 *   Error code and message.
 *
 * Output Format:
 *   out_ciphertext contains: [nonce(12 bytes) | ciphertext | auth_tag(16 bytes)]
 *   Total output size = plaintext_len + 28 bytes
 *
 * Requirements:
 *   - out_ciphertext_capacity >= plaintext_len + 28
 *   - out_ciphertext_len must be non-NULL pointer
 *   - AES key must be set via environment variable CENTRA_NF_AES_KEY (32 bytes)
 *
 * Example:
 *   unsigned char plaintext[] = "Secret message";
 *   unsigned char ciphertext[128];  // Sufficient for example
 *   size_t ciphertext_len = 0;
 *
 *   cnf_error_t err = cnf_aes256_encrypt(
 *       plaintext, sizeof(plaintext),
 *       ciphertext, sizeof(ciphertext),
 *       &ciphertext_len
 *   );
 *   if (err.code != 0) {
 *       fprintf(stderr, "Encryption failed: %s\n", err.message);
 *       cnf_free_error(&err);
 *   }
 */
cnf_error_t cnf_aes256_encrypt(
    const uint8_t *plaintext,
    size_t plaintext_len,
    uint8_t *out_ciphertext,
    size_t out_ciphertext_capacity,
    size_t *out_ciphertext_len
);

/**
 * Decrypt data using AES-256-GCM.
 *
 * Args:
 *   ciphertext               - Encrypted data (format: [nonce(12) | ciphertext | tag(16)])
 *   ciphertext_len           - Length of ciphertext
 *   out_plaintext            - Output buffer for decrypted data
 *   out_plaintext_capacity   - Size of output buffer
 *   out_plaintext_len        - Output: actual size written
 *
 * Returns:
 *   Error code and message.
 *   CNF_CRYPTO_ERROR if decryption fails or authentication tag is invalid.
 *
 * Requirements:
 *   - ciphertext_len >= 28 (minimum: 12-byte nonce + 16-byte tag)
 *   - out_plaintext_capacity >= ciphertext_len - 28
 *   - out_plaintext_len must be non-NULL
 *
 * Note:
 *   Decryption fails if authentication tag is invalid (data was tampered).
 *   This is a security feature.
 *
 * Example:
 *   unsigned char plaintext[64];
 *   size_t plaintext_len = 0;
 *
 *   cnf_error_t err = cnf_aes256_decrypt(
 *       ciphertext, ciphertext_len,
 *       plaintext, sizeof(plaintext),
 *       &plaintext_len
 *   );
 *   if (err.code == CNF_CRYPTO_ERROR) {
 *       fprintf(stderr, "Decryption failed - data was tampered\n");
 *   }
 */
cnf_error_t cnf_aes256_decrypt(
    const uint8_t *ciphertext,
    size_t ciphertext_len,
    uint8_t *out_plaintext,
    size_t out_plaintext_capacity,
    size_t *out_plaintext_len
);

/* ============================================================================
 * ERROR MANAGEMENT
 * ============================================================================ */

/**
 * Free error message string.
 *
 * Args:
 *   err - Pointer to CnfError (can be NULL, which is safe)
 *
 * Note: Only call if error.message was allocated by CENTRA-NF library.
 *       Call once per error. Pointer becomes invalid after.
 *
 * Example:
 *   cnf_error_t err = cnf_compile(...);
 *   if (err.code != 0) {
 *       printf("Error: %s\n", err.message);
 *       cnf_free_error(&err);  // MUST call this
 *   }
 */
void cnf_free_error(cnf_error_t *err);

/* ============================================================================
 * LIBRARY MANAGEMENT
 * ============================================================================ */

/**
 * Get CENTRA-NF library version string.
 *
 * Returns:
 *   Pointer to static string (e.g., "CENTRA-NF 1.0.0")
 *   Pointer is valid for program lifetime (no need to free).
 *
 * Example:
 *   printf("Version: %s\n", cnf_version());
 *   // Output: Version: CENTRA-NF 1.0.0
 */
const char *cnf_version(void);

/**
 * Initialize CENTRA-NF library.
 *
 * Returns:
 *   Error code. Success = 0.
 *   Can be called multiple times (idempotent).
 *
 * Note: Currently always succeeds. Reserved for future initialization.
 *
 * Example:
 *   cnf_error_t err = cnf_init();
 *   if (err.code != 0) {
 *       fprintf(stderr, "Initialization failed\n");
 *       return 1;
 *   }
 */
cnf_error_t cnf_init(void);

/* ============================================================================
 * C++ CONVENIENCE WRAPPERS (optional)
 *
 * These are only included if __cplusplus is defined.
 * Provides RAII-style resource management for C++ users.
 * ============================================================================ */

#ifdef __cplusplus

namespace centra_nf {

/**
 * C++ Exception class for CENTRA-NF errors.
 */
class CnfException : public std::exception {
public:
    CnfException(int code, const std::string &msg)
        : code_(code), message_(msg) {}

    int code() const { return code_; }
    const char *what() const noexcept override { return message_.c_str(); }

private:
    int code_;
    std::string message_;
};

/**
 * C++ RAII wrapper for cnf_program_t.
 * Automatically frees program on destruction.
 */
class Program {
public:
    Program() : handle_(nullptr) {}

    ~Program() {
        if (handle_) cnf_free_program(handle_);
    }

    Program(const Program &) = delete;
    Program &operator=(const Program &) = delete;

    Program(Program &&other) noexcept : handle_(other.handle_) {
        other.handle_ = nullptr;
    }

    Program &operator=(Program &&other) noexcept {
        if (this != &other) {
            if (handle_) cnf_free_program(handle_);
            handle_ = other.handle_;
            other.handle_ = nullptr;
        }
        return *this;
    }

    static Program compile(const std::string &source) {
        Program prog;
        cnf_error_t err = cnf_compile(source.c_str(), &prog.handle_);
        if (err.code != 0) {
            std::string msg(err.message ? err.message : "unknown error");
            cnf_free_error(&err);
            throw CnfException(err.code, msg);
        }
        return prog;
    }

    cnf_program_t *get() const { return handle_; }

private:
    cnf_program_t *handle_;
};

/**
 * C++ RAII wrapper for cnf_runtime_t.
 * Automatically frees runtime on destruction.
 */
class Runtime {
public:
    Runtime() : handle_(cnf_create_runtime()) {
        if (!handle_) throw std::bad_alloc();
    }

    ~Runtime() {
        if (handle_) cnf_free_runtime(handle_);
    }

    Runtime(const Runtime &) = delete;
    Runtime &operator=(const Runtime &) = delete;

    Runtime(Runtime &&other) noexcept : handle_(other.handle_) {
        other.handle_ = nullptr;
    }

    Runtime &operator=(Runtime &&other) noexcept {
        if (this != &other) {
            if (handle_) cnf_free_runtime(handle_);
            handle_ = other.handle_;
            other.handle_ = nullptr;
        }
        return *this;
    }

    void execute(const Program &program) {
        cnf_error_t err = cnf_execute(handle_, program.get());
        if (err.code != 0) {
            std::string msg(err.message ? err.message : "execution failed");
            cnf_free_error(&err);
            throw CnfException(err.code, msg);
        }
    }

    cnf_runtime_t *get() const { return handle_; }

private:
    cnf_runtime_t *handle_;
};

}  // namespace centra_nf

#endif  // __cplusplus

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // CENTRA_NF_H
