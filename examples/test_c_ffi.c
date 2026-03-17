/*
 * CENTRA-NF C FFI Example
 * 
 * Tests core FFI functions: SHA-256, encryption, and decryption
 * Compile: gcc -o test_c_ffi examples/test_c_ffi.c -L target/release -lcentra_nf -lm
 * Run: ./test_c_ffi
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "../centra_nf.h"

#define TEST_DATA "Hello, CENTRA-NF!"
#define TEST_DATA_LEN 18
#define AES_KEY "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

void print_separator(const char* title) {
    printf("\n%s\n", "====================================");
    printf("%s\n", title);
    printf("%s\n", "====================================");
}

void print_bytes(const uint8_t* data, size_t len, const char* prefix) {
    printf("%s: ", prefix);
    for (size_t i = 0; i < len && i < 32; i++) {
        printf("%02x", data[i]);
    }
    if (len > 32) printf("...");
    printf(" (%zu bytes)\n", len);
}

int test_sha256() {
    print_separator("Test 1: SHA-256 Hashing");
    
    // Allocate buffer for 64-char hash + null terminator
    const size_t hash_buf_size = 65;
    char hash_buf[hash_buf_size];
    
    // Call FFI function
    CnfError err = cnf_sha256((const uint8_t*)TEST_DATA, TEST_DATA_LEN, 
                              hash_buf, hash_buf_size);
    
    if (err.code != 0) {
        printf("❌ SHA-256 failed: code=%d msg=%s\n", err.code, err.message ? err.message : "");
        if (err.message) {
            cnf_free_error(&err);
        }
        return 0;
    }
    
    printf("✅ SHA-256 computed successfully\n");
    printf("   Input: \"%s\" (%d bytes)\n", TEST_DATA, TEST_DATA_LEN);
    printf("   Hash: %s\n", hash_buf);
    printf("   Length: %zu\n", strlen(hash_buf));
    
    // Verify hash is 64 characters (SHA-256 in hex)
    if (strlen(hash_buf) != 64) {
        printf("❌ Hash length invalid: expected 64, got %zu\n", strlen(hash_buf));
        return 0;
    }
    
    return 1;
}

int test_encryption() {
    print_separator("Test 2: AES-256-GCM Encryption");
    
    // Set AES key environment variable
    setenv("CENTRA_NF_AES_KEY", AES_KEY, 1);
    
    const char* plaintext = "Secret message for encryption test";
    size_t plaintext_len = strlen(plaintext);
    
    // Allocate buffer for ciphertext (nonce + encrypted data)
    // Typically: 12 bytes (nonce) + plaintext_len + 16 bytes (tag) = plaintext_len + 28
    size_t ciphertext_capacity = plaintext_len + 64;
    uint8_t* ciphertext = (uint8_t*)malloc(ciphertext_capacity);
    size_t ciphertext_len = 0;
    
    // Call FFI encryption
    CnfError err = cnf_aes256_encrypt((const uint8_t*)plaintext, plaintext_len,
                                      ciphertext, ciphertext_capacity, &ciphertext_len);
    
    if (err.code != 0) {
        printf("❌ Encryption failed: code=%d msg=%s\n", err.code, err.message ? err.message : "");
        if (err.message) {
            cnf_free_error(&err);
        }
        free(ciphertext);
        return 0;
    }
    
    printf("✅ Encryption successful\n");
    printf("   Plaintext: \"%s\" (%zu bytes)\n", plaintext, plaintext_len);
    printf("   Ciphertext: %zu bytes\n", ciphertext_len);
    print_bytes(ciphertext, ciphertext_len, "   Data");
    
    // Now decrypt
    print_separator("Test 3: AES-256-GCM Decryption");
    
    uint8_t* decrypted = (uint8_t*)malloc(plaintext_len + 1);
    size_t decrypted_len = 0;
    
    // Call FFI decryption
    err = cnf_aes256_decrypt(ciphertext, ciphertext_len,
                             decrypted, plaintext_len, &decrypted_len);
    
    free(ciphertext);
    
    if (err.code != 0) {
        printf("❌ Decryption failed: code=%d msg=%s\n", err.code, err.message ? err.message : "");
        if (err.message) {
            cnf_free_error(&err);
        }
        free(decrypted);
        return 0;
    }
    
    printf("✅ Decryption successful\n");
    printf("   Ciphertext length: %zu bytes\n", ciphertext_len);
    printf("   Decrypted length: %zu bytes\n", decrypted_len);
    
    // Verify round-trip
    decrypted[decrypted_len] = '\0';
    if (decrypted_len == plaintext_len && memcmp(decrypted, plaintext, plaintext_len) == 0) {
        printf("   Decrypted: \"%s\"\n", (char*)decrypted);
        printf("✅ Round-trip verification PASSED\n");
        free(decrypted);
        return 1;
    } else {
        printf("❌ Round-trip verification FAILED\n");
        printf("   Expected: %s\n", plaintext);
        printf("   Got: %s\n", (char*)decrypted);
        free(decrypted);
        return 0;
    }
}

int main() {
    printf("====================================\n");
    printf("CENTRA-NF C FFI Integration Tests\n");
    printf("====================================\n");
    
    int passed = 0;
    int failed = 0;
    
    // Test 1: SHA-256
    if (test_sha256()) {
        passed++;
    } else {
        failed++;
    }
    
    // Test 2 & 3: Encryption and Decryption
    if (test_encryption()) {
        passed++;
    } else {
        failed++;
    }
    
    // Summary
    print_separator("Test Summary");
    printf("Passed: %d\n", passed);
    printf("Failed: %d\n", failed);
    
    if (failed == 0) {
        printf("\n✅ ALL TESTS PASSED\n");
        return 0;
    } else {
        printf("\n❌ SOME TESTS FAILED\n");
        return 1;
    }
}
