
# CENTRA-NF Security Model

## Assets
- AES-256 keys (`CENTRA_NF_AES_KEY`, managed via KeyManager)
- Quantum key pairs (Kyber, Dilithium, SPHINCS+) — ZeroizeOnDrop
- Cluster auth tokens (`CENTRA_NF_CLUSTER_TOKEN`)
- Governance audit logs (append-only)

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