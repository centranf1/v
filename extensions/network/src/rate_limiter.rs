//! # Token bucket rate limiter per remote node
//!
//! Implements per-node rate limiting with configurable token generation rate.
//! Prevents denial-of-service attacks through controlled message acceptance.

use std::collections::HashMap;
use std::time::Instant;
use crate::error::CnfNetworkError;
use crate::vector_clock::NodeId;


/// Token bucket per node
struct Bucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,    // max token
    rate: f64,        // token per detik
}


impl Bucket {
    fn new(capacity: f64, rate: f64) -> Self {
        Self { tokens: capacity, last_refill: Instant::now(), capacity, rate }
    }


    fn try_consume(&mut self, cost: f64) -> bool {
        // Refill berdasarkan waktu yang berlalu
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_refill = Instant::now();
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }
}


pub struct RateLimiter {
    buckets: HashMap<NodeId, Bucket>,
    default_capacity: f64,
    default_rate: f64,
}


impl RateLimiter {
    /// capacity: max burst, rate: pesan/detik steady-state
    pub fn new(capacity: f64, rate: f64) -> Self {
        Self { buckets: HashMap::new(), default_capacity: capacity, default_rate: rate }
    }


    /// Cek apakah pesan dari node_id boleh diproses
    pub fn check(&mut self, node_id: &NodeId) -> Result<(), CnfNetworkError> {
        let cap = self.default_capacity;
        let rate = self.default_rate;
        let bucket = self.buckets.entry(node_id.clone())
            .or_insert_with(|| Bucket::new(cap, rate));
        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            Err(CnfNetworkError::RateLimitExceeded(node_id.clone()))
        }
    }


    /// Set limit khusus per node (misal: node terpercaya dapat limit lebih tinggi)
    pub fn set_node_limit(&mut self, node_id: NodeId, capacity: f64, rate: f64) {
        self.buckets.insert(node_id, Bucket::new(capacity, rate));
    }
}
