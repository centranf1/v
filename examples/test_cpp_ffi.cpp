/*
 * CENTRA-NF C++ Wrapper Example
 * 
 * Object-oriented C++ interface to CENTRA-NF FFI layer
 * Compile: g++ -std=c++17 -o test_cpp_ffi examples/test_cpp_ffi.cpp -L target/release -lcentra_nf -lm
 * Run: ./test_cpp_ffi
 */

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <memory>
#include <stdexcept>
#include <iomanip>
#include <sstream>

extern "C" {
    #include "../centra_nf.h"
}

// ============================================================================
// C++ Wrapper Classes for CENTRA-NF FFI
// ============================================================================

class CnfException : public std::runtime_error {
public:
    explicit CnfException(const CnfError& err)
        : std::runtime_error(format_error(err))
    {
        if (err.message) {
            cnf_free_error(const_cast<CnfError*>(&err));
        }
    }
    
private:
    static std::string format_error(const CnfError& err) {
        std::ostringstream oss;
        oss << "CENTRA-NF Error (code=" << err.code << "): ";
        if (err.message) {
            oss << err.message;
        } else {
            oss << "Unknown error";
        }
        return oss.str();
    }
};

class CnfCrypto {
public:
    // Compute SHA-256 hash of data
    static std::string sha256(const std::vector<uint8_t>& data) {
        const size_t hash_buf_size = 65;
        char hash_buf[hash_buf_size];
        
        CnfError err = cnf_sha256(data.data(), data.size(), hash_buf, hash_buf_size);
        if (err.code != 0) {
            throw CnfException(err);
        }
        
        return std::string(hash_buf);
    }
    
    // Compute SHA-256 hash of C string
    static std::string sha256(const std::string& str) {
        std::vector<uint8_t> data(str.begin(), str.end());
        return sha256(data);
    }
    
    // Encrypt data using AES-256-GCM
    static std::vector<uint8_t> encrypt(const std::vector<uint8_t>& plaintext) {
        // Allocate space for: 12-byte nonce + ciphertext + 16-byte tag
        std::vector<uint8_t> ciphertext(plaintext.size() + 64);
        size_t ciphertext_len = 0;
        
        CnfError err = cnf_aes256_encrypt(plaintext.data(), plaintext.size(),
                                          ciphertext.data(), ciphertext.size(),
                                          &ciphertext_len);
        if (err.code != 0) {
            throw CnfException(err);
        }
        
        ciphertext.resize(ciphertext_len);
        return ciphertext;
    }
    
    // Encrypt C string
    static std::vector<uint8_t> encrypt(const std::string& plaintext) {
        std::vector<uint8_t> data(plaintext.begin(), plaintext.end());
        return encrypt(data);
    }
    
    // Decrypt data using AES-256-GCM
    static std::vector<uint8_t> decrypt(const std::vector<uint8_t>& ciphertext) {
        std::vector<uint8_t> plaintext(ciphertext.size());
        size_t plaintext_len = 0;
        
        CnfError err = cnf_aes256_decrypt(ciphertext.data(), ciphertext.size(),
                                          plaintext.data(), plaintext.size(),
                                          &plaintext_len);
        if (err.code != 0) {
            throw CnfException(err);
        }
        
        plaintext.resize(plaintext_len);
        return plaintext;
    }
};

// ============================================================================
// Helper Functions
// ============================================================================

void print_separator(const std::string& title) {
    std::printf("\n%s\n", std::string(60, '=').c_str());
    std::printf("%s\n", title.c_str());
    std::printf("%s\n", std::string(60, '=').c_str());
}

std::string bytes_to_hex(const std::vector<uint8_t>& bytes, size_t max_display = 32) {
    std::ostringstream oss;
    for (size_t i = 0; i < bytes.size() && i < max_display; i++) {
        oss << std::hex << std::setw(2) << std::setfill('0') << (int)bytes[i];
    }
    if (bytes.size() > max_display) {
        oss << "...";
    }
    return oss.str();
}

std::string bytes_to_string(const std::vector<uint8_t>& bytes) {
    return std::string(bytes.begin(), bytes.end());
}

// ============================================================================
// Test Functions
// ============================================================================

bool test_sha256() {
    print_separator("Test 1: SHA-256 Hashing (C++)");
    
    try {
        std::string test_data = "Hello, CENTRA-NF from C++!";
        std::string hash = CnfCrypto::sha256(test_data);
        
        std::printf("✅ SHA-256 computed successfully\n");
        std::printf("   Input: \"%s\" (%zu bytes)\n", test_data.c_str(), test_data.size());
        std::printf("   Hash: %s\n", hash.c_str());
        std::printf("   Length: %zu characters\n", hash.size());
        
        if (hash.size() != 64) {
            std::printf("❌ Invalid hash length: expected 64, got %zu\n", hash.size());
            return false;
        }
        
        return true;
    } catch (const std::exception& e) {
        std::printf("❌ Exception: %s\n", e.what());
        return false;
    }
}

bool test_encryption_decryption() {
    print_separator("Test 2: AES-256-GCM Encryption & Decryption (C++)");
    
    try {
        // Set AES key
        ::setenv("CENTRA_NF_AES_KEY", 
                 "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef", 1);
        
        std::string plaintext = "Sensitive data protected by CENTRA-NF encryption!";
        std::printf("   Input: \"%s\" (%zu bytes)\n", plaintext.c_str(), plaintext.size());
        
        // Encrypt
        auto ciphertext = CnfCrypto::encrypt(plaintext);
        std::printf("✅ Encryption successful\n");
        std::printf("   Ciphertext: %zu bytes\n", ciphertext.size());
        std::printf("   Data: %s\n", bytes_to_hex(ciphertext).c_str());
        
        // Decrypt
        auto decrypted = CnfCrypto::decrypt(ciphertext);
        std::string decrypted_str = bytes_to_string(decrypted);
        std::printf("✅ Decryption successful\n");
        std::printf("   Decrypted: \"%s\" (%zu bytes)\n", decrypted_str.c_str(), decrypted.size());
        
        // Verify round-trip
        if (decrypted_str == plaintext) {
            std::printf("✅ Round-trip verification PASSED\n");
            return true;
        } else {
            std::printf("❌ Round-trip verification FAILED\n");
            std::printf("   Expected: %s\n", plaintext.c_str());
            std::printf("   Got: %s\n", decrypted_str.c_str());
            return false;
        }
    } catch (const std::exception& e) {
        std::printf("❌ Exception: %s\n", e.what());
        return false;
    }
}

bool test_multiple_operations() {
    print_separator("Test 3: Multiple Independent Operations (C++)");
    
    try {
        ::setenv("CENTRA_NF_AES_KEY",
                 "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210", 1);
        
        // Multiple hash operations
        std::vector<std::string> test_strings = {
            "CENTRA-NF v1.0.0",
            "Production Ready",
            "Cryptography Test"
        };
        
        std::printf("   Computing hashes:\n");
        for (const auto& str : test_strings) {
            std::string hash = CnfCrypto::sha256(str);
            std::printf("   • \"%s\" → %s...\n", str.c_str(), hash.substr(0, 16).c_str());
        }
        
        // Encrypt multiple messages with same key
        std::printf("\n   Encrypting multiple messages:\n");
        std::vector<std::vector<uint8_t>> ciphertexts;
        
        std::vector<std::string> messages = {
            "Message 1",
            "Message 2",
            "Message 3"
        };
        
        for (const auto& msg : messages) {
            auto cipher = CnfCrypto::encrypt(msg);
            ciphertexts.push_back(cipher);
            std::printf("   • \"%s\" → %zu bytes (nonce + ciphertext + tag)\n", 
                       msg.c_str(), cipher.size());
        }
        
        // Decrypt all messages
        std::printf("\n   Decrypting all messages:\n");
        for (size_t i = 0; i < ciphertexts.size(); i++) {
            auto plain = CnfCrypto::decrypt(ciphertexts[i]);
            std::string plain_str = bytes_to_string(plain);
            
            if (plain_str == messages[i]) {
                std::printf("   ✓ Message %zu: \"%s\" (verified)\n", i+1, plain_str.c_str());
            } else {
                std::printf("   ✗ Message %zu: verification failed\n", i+1);
                return false;
            }
        }
        
        std::printf("✅ All operations successful\n");
        return true;
    } catch (const std::exception& e) {
        std::printf("❌ Exception: %s\n", e.what());
        return false;
    }
}

// ============================================================================
// Main
// ============================================================================

int main() {
    std::printf("============================================================\n");
    std::printf("CENTRA-NF C++ Wrapper Integration Tests\n");
    std::printf("============================================================\n");
    
    int passed = 0;
    int failed = 0;
    
    // Run tests
    if (test_sha256()) {
        passed++;
    } else {
        failed++;
    }
    
    if (test_encryption_decryption()) {
        passed++;
    } else {
        failed++;
    }
    
    if (test_multiple_operations()) {
        passed++;
    } else {
        failed++;
    }
    
    // Summary
    print_separator("Test Summary");
    std::printf("Passed: %d\n", passed);
    std::printf("Failed: %d\n", failed);
    std::printf("Total:  %d\n", passed + failed);
    
    if (failed == 0) {
        std::printf("\n✅ ALL TESTS PASSED\n");
        return 0;
    } else {
        std::printf("\n❌ SOME TESTS FAILED\n");
        return 1;
    }
}
