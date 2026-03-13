
# CENTRA-NF Security Model

## Assets
- AES-256 keys (`CENTRA_NF_AES_KEY`, managed via KeyManager)
- Quantum key pairs (Kyber, Dilithium, SPHINCS+) — ZeroizeOnDrop
- Cluster auth tokens (`CENTRA_NF_CLUSTER_TOKEN`)
- Governance audit logs (append-only)

## AES Key Setup (cnf-security)
- `CENTRA_NF_AES_KEY` must be a 64-character hex string (32 bytes key).
- Do NOT use raw ASCII passwords or human-readable secrets (low entropy).

### Generate key (example)
```bash
openssl rand -hex 32 | tr -d '\n' > .env.key
export CENTRA_NF_AES_KEY=$(cat .env.key)
```

- If key is absent, `KeyError::KeyMissing` is returned.
- If key length is not 64 or format invalid, `KeyError::KeyInvalid` is returned.

## Threat Mitigations
| Threat | Mitigation |
|--------|-----------|
| Network eavesdropping | HMAC 3-way mutual auth; TLS planned v1.2 |
| Timing attack on HMAC | `subtle::ConstantTimeEq` (Gate 18) |
| Key compromise | Key rotation via KeyManager; ZeroizeOnDrop |
| Runtime crash via input | panic sweep → Result<> (Gate 17) |
| Malicious frame injection | CRC32 frame integrity + HMAC auth |
| Supply chain | SBOM (Gate 20) + cargo-audit (Gate 21) |

## Crypto Primitives
- Symmetric: AES-256-GCM, random nonce (OsRng)
- Hash: SHA-256 (data integrity), HMAC-SHA256 (auth)
- Post-quantum: ML-KEM-768, ML-DSA-65, SLH-DSA-SHAKE-256f (NIST PQC)

## Out of Scope (Current)
- TLS/mTLS channel encryption → v1.2.0
- HSM/KMS integration → v1.3.0
- FIPS 140-3 lab validation → roadmap

## Reporting
security@centra-nf.internal | SLA: 72h critical, 7d high