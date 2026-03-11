# CENTRA-NF FIPS Alignment

## FIPS 140-3 (Cryptographic Module)
| Requirement | Status | Notes |
|-------------|--------|-------|
| SP 800-90A DRBG | ⚠️ Planned | Current: OsRng (tidak deterministic). Target v2.0: CTR-DRBG |
| SP 800-57 Key Management | ✅ Partial | KeyManager dengan ZeroizeOnDrop dan rotation |
| AES-256-GCM | ✅ Active | Via aes-gcm crate, random nonce OsRng |
| HMAC-SHA256 | ✅ Active | Constant-time via subtle crate |
| Post-Quantum (FIPS 203/204/205) | ✅ Active | ML-KEM-768, ML-DSA-65, SLH-DSA via pqcrypto |

## Gaps untuk Sertifikasi Lab NVLAP
- OsRng harus diganti CTR-DRBG (SP 800-90A) untuk mode FIPS
- HSM/PKCS#11 integration diperlukan (target v1.3.0)
- Physical security boundary documentation (target v2.0.0)
- Power-up self-test untuk crypto module (target v2.0.0)
