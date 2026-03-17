//! # Connection pool manager with idle timeout
//!
//! Manages a pool of TCP connections to remote nodes.
//! Automatically closes idle connections after 30 seconds to prevent resource leaks.

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::error::CnfNetworkError;
use crate::vector_clock::NodeId;


const MAX_IDLE_MS: u64 = 30_000;   // tutup koneksi idle > 30 detik
const MAX_POOL_SIZE: usize = 8;     // max koneksi per node


struct PoolEntry {
    stream: TcpStream,
    last_used: Instant,
    in_use: bool,
}


pub struct ConnectionPool {
    pools: Arc<Mutex<HashMap<NodeId, Vec<PoolEntry>>>>,
    max_per_node: usize,
}


impl ConnectionPool {
    pub fn new() -> Self {
        Self { pools: Arc::new(Mutex::new(HashMap::new())), max_per_node: MAX_POOL_SIZE }
    }


    /// Ambil koneksi dari pool — atau buat baru jika tidak ada
    pub fn acquire(&self, node_id: &NodeId, addr: &str) -> Result<TcpStream, CnfNetworkError> {
        let mut pools = self.pools.lock().map_err(|_| CnfNetworkError::LockPoisoned)?;
        let entries = pools.entry(node_id.clone()).or_default();
        // Cari entry idle yang masih segar
        for e in entries.iter_mut() {
            if !e.in_use && e.last_used.elapsed().as_millis() < MAX_IDLE_MS as u128 {
                e.in_use = true;
                // Clone stream via try_clone untuk reuse
                return e.stream.try_clone().map_err(|e| CnfNetworkError::ConnectionFailed(e.to_string()));
            }
        }
        // Tidak ada — buat koneksi baru
        if entries.len() >= self.max_per_node {
            return Err(CnfNetworkError::ConnectionFailed("Pool exhausted".into()));
        }
        let stream = TcpStream::connect(addr)
            .map_err(|e| CnfNetworkError::ConnectionFailed(e.to_string()))?;
        stream.set_nodelay(true).ok();  // disable Nagle — penting untuk latensi
        Ok(stream)
    }


    /// Bersihkan koneksi idle
    pub fn evict_idle(&self) -> Result<usize, CnfNetworkError> {
        let mut pools = self.pools.lock().map_err(|_| CnfNetworkError::LockPoisoned)?;
        let mut count = 0usize;
        for entries in pools.values_mut() {
            let before = entries.len();
            entries.retain(|e| !e.in_use && e.last_used.elapsed().as_millis() < MAX_IDLE_MS as u128);
            count += before - entries.len();
        }
        Ok(count)
    }
}
