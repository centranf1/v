//! Network layer errors
//!
//! Layer L6 error codes for distributed operations.
//! All errors are explicit and cite context.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CnfNetworkError {
    /// L6.001: Connection to remote node failed
    #[error("L6.001 ConnectionFailed: {0}")]
    ConnectionFailed(String),

    /// L6.002: Failed to send message to target
    #[error("L6.002 SendFailed: {0}")]
    SendFailed(String),

    /// L6.003: Receive operation timed out
    #[error("L6.003 ReceiveTimeout: {0}")]
    ReceiveTimeout(String),

    /// L6.004: Checksum validation failed (corruption detected)
    #[error("L6.004 ChecksumMismatch: expected {expected}, got {received}")]
    ChecksumMismatch { expected: u32, received: u32 },

    /// L6.005: Circuit breaker is open
    #[error("L6.005 CircuitOpen: breaker open for {0}")]
    CircuitOpen(String),

    /// L6.006: Target node not found in registry
    #[error("L6.006 NodeNotFound: {0}")]
    NodeNotFound(String),

    /// L6.007: Message serialization failed
    #[error("L6.007 SerializationFailed: {0}")]
    SerializationFailed(String),

    /// L6.008: Failover attempt failed
    #[error("L6.008 FailoverFailed: {0}")]
    FailoverFailed(String),

    /// L6.009: Authentication failed (PSK handshake rejected)
    #[error("L6.009 AuthenticationFailed: PSK handshake rejected")]
    AuthenticationFailed,

    /// L6.010: Frame size exceeds configured maximum
    #[error("L6.010 FrameTooLarge: {0} bytes exceeds limit")]
    FrameTooLarge(usize),

    /// L6.011: Mutex lock is poisoned (thread panic detected)
    #[error("L6.011 LockPoisoned: mutex lock poisoned")]
    LockPoisoned,

    /// L6.012: TLS configuration or handshake failed
    #[error("L6.012 TlsError: {0}")]
    TlsError(String),
}
